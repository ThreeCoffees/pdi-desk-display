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
    request::{Method},
};
use static_cell::StaticCell;

const EXTERNAL_WIFI_SSID: &str = env!("EXTERNAL_WIFI_SSID");
const EXTERNAL_WIFI_PASSWORD: &str = env!("EXTERNAL_WIFI_PASSWORD");

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

static AP_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static STA_RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();

static TCP_CLIENT_STATE: StaticCell<TcpClientState<1, 1500, 1500>> = StaticCell::new();

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

    // Wait for the AP to become operational. Do not block on external Wi-Fi.
    ap_stack.wait_config_up().await;

    info!("AP `{}` ready at http://192.168.2.1:8080/", WIFI_SSID);

    spawner.spawn(wifi_connection_loop(ap_stack).unwrap());

    spawner
        .spawn(
            sta_http_poll_task(sta_stack, "http://httpbin.org/get?hello=Hello+esp-hal", 60)
            .unwrap(),
        );
}

#[embassy_executor::task]
async fn wifi_connection_loop(ap_stack: Stack<'static>) {
    use embedded_io_async::Write;

    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];

    let mut socket = TcpSocket::new(ap_stack, &mut rx_buffer, &mut tx_buffer);

    socket.set_timeout(Some(Duration::from_secs(10)));

    loop {
        info!("AP: waiting for HTTP connection...");

        if let Err(e) = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 8080,
            })
            .await
        {
            info!("AP accept error: {:?}", e);
            socket.abort();
            continue;
        }

        info!("AP: client connected");

        let mut request = [0u8; 1024];
        let mut pos = 0;

        loop {
            if pos == request.len() {
                break;
            }

            match socket.read(&mut request[pos..]).await {
                Ok(0) => break,
                Ok(n) => {
                    pos += n;

                    if request[..pos].windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(e) => {
                    info!("AP read error: {:?}", e);
                    break;
                }
            }
        }

        if pos > 0 {
            match core::str::from_utf8(&request[..pos]) {
                Ok(request) => info!("AP request: {}", request),
                Err(_) => info!("AP received invalid UTF-8"),
            }
        }

        let response = b"HTTP/1.1 200 OK\r\n\
            Content-Type: text/html; charset=utf-8\r\n\
            Connection: close\r\n\
            Content-Length: 70\r\n\
            \r\n\
            <html><body><h1>Hello from ESP32-C3!</h1></body></html>";

        if let Err(e) = socket.write_all(response).await {
            info!("AP write error: {:?}", e);
        }

        let _ = socket.flush().await;
        socket.close();
        socket.abort();
    }
}

#[embassy_executor::task]
async fn sta_http_poll_task(
    sta_stack: Stack<'static>,
    url: &'static str,
    interval_secs: u64,
) {
    let tcp_client = TcpClient::new(
        sta_stack,
        TCP_CLIENT_STATE.init(
            TcpClientState::<1, 1500, 1500>::new(),
        ),
    );

    let dns_client = DnsSocket::new(sta_stack);

    let mut rx_buffer = [0u8; 4096];

    loop {
        if !sta_stack.is_link_up() || sta_stack.config_v4().is_none() {
            info!("Station is not connected; waiting...");
            Timer::after(Duration::from_secs(5)).await;
            continue;
        }

        info!("Sending HTTP GET to {}", url);

        let mut client = HttpClient::new(&tcp_client, &dns_client);

        match client.request(Method::GET, url).await {
            Ok(mut builder) => {
                match builder.send(&mut rx_buffer).await {
                    Ok(response) => {
                        info!("HTTP request succeeded");

                        let mut body = response.body().reader();
                        let mut chunk = [0u8; 256];

                        loop {
                            match body.read(&mut chunk).await {
                                Ok(0) => break,
                                Ok(n) => {
                                    // Process the response body here.
                                    info!("Received {} response bytes", n);
                                }
                                Err(e) => {
                                    info!(
                                        "HTTP response read error: {:?}",
                                        defmt::Debug2Format(&e)
                                    );
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        info!(
                            "HTTP request failed: {:?}",
                            defmt::Debug2Format(&e)
                        );
                    }
                }
            }
            Err(e) => {
                info!(
                    "HTTP DNS/connect error: {:?}",
                    defmt::Debug2Format(&e)
                );
            }
        }

        Timer::after(Duration::from_secs(interval_secs)).await;
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
