use defmt::error;
use defmt::info;
use embassy_futures::select::select;
use embassy_net::{
    Runner, Stack,
    dns::DnsSocket,
    tcp::{TcpSocket, client::TcpClient},
};
use embassy_time::{Duration, Timer};
use embedded_io_async::Read;
use esp_radio::wifi::Interface;

use esp_radio::wifi::{
    AuthenticationMethodConfig, Config, WifiController, ap::AccessPointConfig, sta::StationConfig,
};

use super::config::WifiCredentials;
use super::*;
use crate::wifi::{config::WIFI_CONFIG_CHANNEL, http::handle_http_connection};

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

        if let Err(err) = handle_http_connection(&mut socket).await {
            error!("{}", err);
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
        if let Err(error) = controller.connect_async().await {
            info!("Failed to connect to Wi-Fi: {:?}", error);

            match select(
                WIFI_CONFIG_CHANNEL.receive(),
                Timer::after(Duration::from_secs(5)),
            )
            .await
            {
                Either::First(credentials) => {
                    if apply_wifi_credentials(&mut controller, credentials) {
                        continue;
                    }
                }
                Either::Second(_) => {}
            }

            continue;
        }
        info!("Wi-Fi connection started");

        if wait_for_connection_event(&mut controller).await {
            continue;
        }
    }
}

/// Waits for a station disconnect, new credentials, or an AP client event.
///
/// Returns `true` when the station configuration changes or the station
/// disconnects, indicating that the connection loop should restart.
async fn wait_for_connection_event(controller: &mut WifiController<'static>) -> bool {
    loop {
        let event = select(
            select(
                WIFI_CONFIG_CHANNEL.receive(),
                controller.wait_for_disconnect_async(),
            ),
            controller.wait_for_access_point_connected_event_async(),
        )
        .await;

        match event {
            Either::First(Either::First(credentials)) => {
                info!("New Wi-Fi credentials received");

                if apply_wifi_credentials(controller, credentials) {
                    return true;
                }

                info!("Keeping current Wi-Fi configuration");
            }

            Either::First(Either::Second(result)) => {
                log_station_disconnect(result);
                return true;
            }

            Either::Second(result) => {
                log_access_point_event(result);
            }
        }
    }
}

fn log_station_disconnect(result: Result<impl defmt::Format, impl defmt::Format>) {
    match result {
        Ok(disconnected) => {
            info!("Station disconnected: {:?}", disconnected);
        }
        Err(error) => {
            error!("Disconnect event error: {:?}", error);
        }
    }
}

fn log_access_point_event(result: Result<esp_radio::wifi::ap::EventInfo, impl defmt::Format>) {
    match result {
        Ok(event) => match event {
            esp_radio::wifi::ap::EventInfo::Connected(info) => {
                info!("AP client connected: {:?}", info);
            }
            esp_radio::wifi::ap::EventInfo::Disconnected(info) => {
                info!("AP client disconnected: {:?}", info);
            }
        },
        Err(error) => {
            info!("AP event error: {:?}", error);
        }
    }
}

#[embassy_executor::task(pool_size = 2)]
pub async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await
}

fn apply_wifi_credentials(
    controller: &mut WifiController<'static>,
    credentials: WifiCredentials,
) -> bool {
    let station_config = StationConfig::default()
        .with_ssid(credentials.ssid.as_str().try_into().unwrap())
        .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
            credentials.password.as_str().try_into().unwrap(),
        ));

    let ap_config = AccessPointConfig::default()
        .with_ssid(WIFI_SSID.try_into().unwrap())
        .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
            WIFI_PASSWORD.try_into().unwrap(),
        ));

    let config = Config::AccessPointStation(station_config, ap_config);

    match controller.set_config(&config) {
        Ok(()) => {
            info!("Wi-Fi configuration updated");
            true
        }
        Err(error) => {
            error!("Failed to update Wi-Fi configuration: {:?}", error);
            false
        }
    }
}
