use defmt::info;
use embassy_net::{Runner, Stack, dns::DnsSocket, tcp::{TcpSocket, client::TcpClient}};
use embassy_time::{Duration, Timer};
use esp_radio::wifi::Interface;
use embedded_io_async::Read;

use crate::wifi::{*};
use crate::wifi::http::handle_http_connection;


#[embassy_executor::task]
pub async fn wifi_connection_loop(ap_stack: Stack<'static>) {
    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];

    let mut socket = TcpSocket::new(ap_stack, &mut rx_buffer, &mut tx_buffer);

    socket.set_timeout(Some(Duration::from_secs(10)));

    loop {
        socket
            .accept(embassy_net::IpListenEndpoint {
                addr: None,
                port: 8080,
            })
            .await;

        if let Err(_err) = handle_http_connection(&mut socket).await {
            // Don't log request contents or credentials.
        }

        socket.close();
        socket.abort();
    }
}

#[embassy_executor::task]
pub async fn sta_http_poll_task(sta_stack: Stack<'static>, url: &'static str, interval_secs: u64) {
    let tcp_client = TcpClient::new(
        sta_stack,
        TCP_CLIENT_STATE.init(TcpClientState::<1, 1500, 1500>::new()),
    );

    let dns_client = DnsSocket::new(sta_stack);

    let mut rx_buffer = [0u8; 4096];

    loop {
        if !sta_stack.is_link_up() || sta_stack.config_v4().is_none() {
            info!("Station is not connected; waiting...");
            Timer::after(Duration::from_secs(5)).await;
            continue;
        }

        let mut tls_rx_buffer = [0u8; 16640];
        let mut tls_tx_buffer = [0u8; 16640];

        let tls_seed = {
            let rng = esp_hal::rng::Rng::new();
            (rng.random() as u64) << 32 | rng.random() as u64
        };

        let tls = TlsConfig::new(
            tls_seed,
            &mut tls_rx_buffer,
            &mut tls_tx_buffer,
            TlsVerify::None,
        );

        let mut client = HttpClient::new_with_tls(&tcp_client, &dns_client, tls);

        info!("Sending HTTP GET to {}", url);

        match client.request(Method::GET, url).await {
            Ok(mut builder) => {
                match builder.send(&mut rx_buffer).await {
                    Ok(response_headers) => {
                        info!("HTTP request succeeded");
                        let mut body = response_headers.body().reader();
                        let mut chunk = [0u8; 256];
                        let mut total_bytes = 0usize;

                        loop {
                            match body.read(&mut chunk).await {
                                Ok(0) => break,
                                Ok(n) => {
                                    total_bytes += n;

                                    // Process the response_headers body here.
                                }
                                Err(e) => {
                                    info!("HTTP response_headers read error: {:?}", e);
                                    break;
                                }
                            }
                        }
                        info!("Total response_headers bytes: {}", total_bytes);
                    }
                    Err(e) => {
                        info!("HTTP request failed: {:?}", e);
                    }
                }
            }
            Err(e) => {
                info!("HTTP DNS/connect error: {:?}", e);
            }
        }

        Timer::after(Duration::from_secs(interval_secs)).await;
    }
}

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
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
pub async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await
}

