use core::net::Ipv4Addr;

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::Either;
use embassy_net::{
    IpListenEndpoint, Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4,
    dns::DnsSocket,
    tcp::{
        TcpSocket,
        client::{TcpClient, TcpClientState},
    },
};
use embassy_time::{Duration, Timer};
use embedded_io_async::Read;
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_radio::wifi::{
    Config, ControllerConfig, Interface, WifiController, ap::AccessPointConfig, sta::StationConfig,
};
use reqwless::{
    client::HttpClient,
    request::{Method, RequestBuilder},
};
use static_cell::StaticCell;

const EXTERNAL_WIFI_SSID: &str = env!("EXTERNAL_WIFI_SSID");
const EXTERNAL_WIFI_PASSWORD: &str = env!("EXTERNAL_WIFI_PASSWORD");

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");


static AP_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static STA_RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();

static TCP_CLIENT_STATE: StaticCell<TcpClientState<1, 1500, 1500>> =
    StaticCell::new();

#[embassy_executor::task]
pub async fn wifi_start_task(spawner: Spawner, wifi: WIFI<'static>) {
    start_wifi(spawner, wifi).await;
}

async fn start_wifi(spawner: Spawner, wifi_peripheral: WIFI<'static>) {
    let access_point_station_config = Config::AccessPointStation(
        StationConfig::default()
            .with_ssid(EXTERNAL_WIFI_SSID.try_into().unwrap())
            .with_authentication(esp_radio::wifi::AuthenticationMethodConfig::Wpa2Personal(
                EXTERNAL_WIFI_PASSWORD.try_into().unwrap(),
            )),
        AccessPointConfig::default()
            .with_ssid(WIFI_SSID.try_into().unwrap())
            .with_authentication(esp_radio::wifi::AuthenticationMethodConfig::Wpa2Personal(
                WIFI_PASSWORD.try_into().unwrap(),
            )),
    );

    info!("Starting wifi");

    let wifi_ap_device = esp_radio::wifi::Interface::access_point();
    let wifi_sta_device = esp_radio::wifi::Interface::station();

    let controller = esp_radio::wifi::WifiController::new(
        wifi_peripheral,
        ControllerConfig::default().with_initial_config(access_point_station_config),
    )
    .unwrap();

    info!("Wifi started");

    let ap_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Addr::new(192, 168, 2, 1), 24),
        gateway: Some(Ipv4Addr::new(192, 168, 2, 1)),
        dns_servers: Default::default(),
    });
    let sta_config = embassy_net::Config::dhcpv4(Default::default());

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let (ap_stack, ap_runner) = embassy_net::new(
        wifi_ap_device,
        ap_config,
        AP_RESOURCES.init(StackResources::<3>::new()),
        seed,
    );
    let (sta_stack, sta_runner) = embassy_net::new(
        wifi_sta_device,
        sta_config,
        STA_RESOURCES.init(StackResources::<4>::new()),
        seed,
    );

    spawner.spawn(connection(controller).unwrap());
    spawner.spawn(net_task(ap_runner).unwrap());
    spawner.spawn(net_task(sta_runner).unwrap());

    let sta_address = loop {
        if let Some(config) = sta_stack.config_v4() {
            let address = config.address.address();
            info!("Got IP: {}", address);
            break address;
        }
        info!("Waiting for IP...");
        Timer::after(Duration::from_millis(500)).await;
    };
    ap_stack.wait_config_up().await;

    info!(
        "Connect to the AP `{}` and point your browser to http://192.168.2.1:8080/",
        WIFI_SSID
    );
    info!("Use a static IP in the range 192.168.2.2 .. 192.168.2.255, use gateway 192.168.2.1");
    info!(
        "Or connect to the ap `{}` and point your browser to http://{}:8080/",
        EXTERNAL_WIFI_SSID, sta_address
    );

    spawner.spawn(wifi_connection_loop(ap_stack, sta_stack).unwrap());
}

#[embassy_executor::task]
async fn wifi_connection_loop(ap_stack: Stack<'static>, sta_stack: Stack<'static>) {
    let tcp_client = TcpClient::new(
        sta_stack,
        TCP_CLIENT_STATE.init(TcpClientState::<1, 1500, 1500>::new()),
    );
    let dns_client = DnsSocket::new(sta_stack);

    let mut ap_server_rx_buffer = [0; 1536];
    let mut ap_server_tx_buffer = [0; 1536];
    let mut sta_server_rx_buffer = [0; 1536];
    let mut sta_server_tx_buffer = [0; 1536];

    let mut ap_server_socket =
        TcpSocket::new(ap_stack, &mut ap_server_rx_buffer, &mut ap_server_tx_buffer);
    ap_server_socket.set_timeout(Some(Duration::from_secs(10)));

    let mut sta_server_socket = TcpSocket::new(
        sta_stack,
        &mut sta_server_rx_buffer,
        &mut sta_server_tx_buffer,
    );
    sta_server_socket.set_timeout(Some(Duration::from_secs(10)));

    loop {
        info!("Wait for connection...");
        let either_socket = embassy_futures::select::select(
            ap_server_socket.accept(IpListenEndpoint {
                addr: None,
                port: 8080,
            }),
            sta_server_socket.accept(IpListenEndpoint {
                addr: None,
                port: 8080,
            }),
        )
        .await;
        let (r, server_socket) = match either_socket {
            Either::First(r) => (r, &mut ap_server_socket),
            Either::Second(r) => (r, &mut sta_server_socket),
        };
        info!("Connected...");

        if let Err(e) = r {
            info!("connect error: {:?}", e);
            continue;
        }

        use embedded_io_async::Write;

        let mut buffer = [0u8; 4092];
        let mut pos = 0;
        loop {
            match server_socket.read(&mut buffer).await {
                Ok(0) => {
                    info!("AP read EOF");
                    break;
                }
                Ok(len) => {
                    let to_print =
                        unsafe { core::str::from_utf8_unchecked(&buffer[..(pos + len)]) };

                    if to_print.contains("\r\n\r\n") {
                        info!("{}", to_print);
                        break;
                    }

                    pos += len;
                }
                Err(e) => {
                    info!("AP read error: {:?}", e);
                    break;
                }
            };
        }
        if sta_stack.is_link_up() {
            info!("connecting via HttpClient...");
            let mut client = HttpClient::new(&tcp_client, &dns_client);
            let mut rx_buf = [0u8; 4096];

            let builder_result = client
                .request(Method::GET, "http://httpbin.org/get?hello=Hello+esp-hal")
                .await;

            match builder_result {
                Ok(req_builder) => {
                    let headers = [("Host", "httpbin.org"), ("Connection", "close")];
                    let mut req_builder = req_builder.headers(&headers);
                    let response_result = req_builder.send(&mut rx_buf).await;

                    match response_result {
                        Ok(response) => {
                            info!("HTTP request successful, streaming body...");

                            let _ = server_socket.write_all(b"HTTP/1.0 200 OK\r\n").await;
                            let _ = server_socket
                                .write_all(b"Content-Type: application/json\r\n")
                                .await;
                            let _ = server_socket.write_all(b"Connection: close\r\n\r\n").await;

                            let mut body_reader = response.body().reader();
                            let mut chunk_buf = [0u8; 1024];

                            loop {
                                match body_reader.read(&mut chunk_buf).await {
                                    Ok(0) => break,
                                    Ok(n) => {
                                        if let Err(e) =
                                            server_socket.write_all(&chunk_buf[..n]).await
                                        {
                                            info!("AP write error: {:?}", e);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        info!("Body read error: {:?}", defmt::Debug2Format(&e));
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            info!("Station request error: {:?}", defmt::Debug2Format(&e));
                            let _ = server_socket
                                .write_all(
                                    b"HTTP/1.0 500 Internal Server Error\r\n\r\nRequest failed",
                                )
                                .await;
                        }
                    }
                }
                Err(e) => {
                    info!("DNS/Connect error: {:?}", defmt::Debug2Format(&e));
                    let _ = server_socket
                        .write_all(b"HTTP/1.0 500 Internal Server Error\r\n\r\nDNS Error")
                        .await;
                }
            }
        } else {
            let r = server_socket
                .write_all(
                    b"HTTP/1.0 200 OK\r\n\r\n\
                    <html>\
                        <body>\
                            <h1>Hello Rust! Hello esp-radio! Station is not connected.</h1>\
                        </body>\
                    </html>\r\n\
                    ",
                )
                .await;
            if let Err(e) = r {
                info!("AP write error: {:?}", e);
            }
        }
        let r = server_socket.flush().await;
        if let Err(e) = r {
            info!("AP flush error: {:?}", e);
        }
        Timer::after(Duration::from_millis(1000)).await;
        server_socket.close();
        Timer::after(Duration::from_millis(1000)).await;
        server_socket.abort();
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    info!("Start connection task");

    loop {
        match controller.connect_async().await {
            Ok(_) => loop {
                let info = embassy_futures::select::select(
                    controller.wait_for_disconnect_async(),
                    controller.wait_for_access_point_connected_event_async(),
                )
                .await;

                match info {
                    Either::First(station_disconnected) => {
                        if let Ok(station_disconnected) = station_disconnected {
                            info!("Station disconnected: {:?}", station_disconnected);
                            break;
                        }
                    }
                    Either::Second(event) => {
                        if let Ok(event) = event {
                            match event {
                                esp_radio::wifi::ap::EventInfo::Connected(info) => {
                                    info!("Station connected: {:?}", info);
                                }
                                esp_radio::wifi::ap::EventInfo::Disconnected(info) => {
                                    info!("Station disconnected: {:?}", info);
                                }
                            }
                        }
                    }
                }
            },
            Err(e) => {
                info!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await
}
