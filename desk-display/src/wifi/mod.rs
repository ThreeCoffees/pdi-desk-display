use core::net::Ipv4Addr;

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::Either;
use embassy_net::{
    Ipv4Cidr, StackResources, StaticConfigV4, tcp::
        client::TcpClientState,
};
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_radio::wifi::{
    Config, WifiController, ap::AccessPointConfig, sta::StationConfig,
};
use esp_radio::wifi::ControllerConfig;
use reqwless::{
    client::{HttpClient, TlsConfig, TlsVerify},
    request::Method,
};
use static_cell::StaticCell;

mod http;
mod tasks;

use tasks::{connection, net_task, sta_http_poll_task, wifi_connection_loop};

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

    spawner.spawn(sta_http_poll_task(sta_stack, "https://news.ycombinator.com/rss", 60).unwrap());
}

