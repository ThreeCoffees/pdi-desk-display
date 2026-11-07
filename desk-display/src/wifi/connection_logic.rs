use defmt::{error, info};
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config, WifiController, ap::AccessPointConfig, sta::StationConfig,
};

use crate::wifi::wifi_config::WifiCredentials;

use super::wifi_config::WIFI_CONFIG_CHANNEL;
use super::*;

pub async fn run(controller: &mut WifiController<'static>) {
    info!("Start connection task");

    loop {
        let credentials = WIFI_CONFIG_CHANNEL.receive().await;

        if credentials.ssid.is_empty() {
            continue;
        }

        if !apply_wifi_credentials(controller, credentials) {
            continue;
        }

        connect_station(controller).await;
    }
}

async fn connect_station(controller: &mut WifiController<'static>) {
    loop {
        if let Err(error) = controller.connect_async().await {
            info!("Failed to connect to Wi-Fi: {:?}", error);

            handle_connection_retry(controller).await;
            continue;
        }

        info!("Wi-Fi connected");

        wait_for_connection_event(controller).await;
    }
}

async fn handle_connection_retry(controller: &mut WifiController<'static>) -> bool {
    match select(
        WIFI_CONFIG_CHANNEL.receive(),
        Timer::after(Duration::from_secs(5)),
    )
    .await
    {
        Either::First(credentials) => {
            if credentials.ssid.is_empty() {
                return false;
            }

            apply_wifi_credentials(controller, credentials)
        }
        Either::Second(_) => false,
    }
}

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
                if credentials.ssid.is_empty() {
                    continue;
                }

                info!("New Wi-Fi credentials received");

                if apply_wifi_credentials(controller, credentials) {
                    return true;
                }
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
