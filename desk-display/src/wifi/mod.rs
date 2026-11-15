use core::cell::UnsafeCell;
use core::net::Ipv4Addr;
use defmt::info;
use embassy_executor::Spawner;
use embassy_net::StaticConfigV4;
use embassy_net::tcp::client::TcpClientState;
use embassy_net::{Ipv4Cidr, Stack, StackResources};
use esp_hal::{peripherals::WIFI, rng::Rng};
use esp_radio::wifi::ControllerConfig;
use esp_radio::wifi::{Config, ap::AccessPointConfig, sta::StationConfig};
use leasehund::DhcpServer;
use static_cell::StaticCell;

mod connection_logic;
mod http;
mod rss;
mod rss_config;
mod tasks;
mod wifi_config;

use tasks::{connection, net_task, sta_http_poll_task, wifi_connection_loop};

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

const AP_IP_STR: &str = env!("AP_IP");
const AP_SUBNET_MASK_STR: &str = env!("AP_SUBNET_MASK");
const AP_GATEWAY_STR: &str = env!("AP_GATEWAY");
const AP_DNS_SERVER_STR: &str = env!("AP_DNS_SERVER");
const DHCP_CLIENT_IP_STR: &str = env!("DHCP_CLIENT_IP");
const DHCP_POOL_END_STR: &str = env!("DHCP_POOL_END");

static AP_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static STA_RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();

/// Wrapper around the TCP client state that is safe to share across tasks.
struct TcpClientStateWrapper {
    inner: UnsafeCell<Option<TcpClientState<1, 1500, 1500>>>,
}

// SAFETY: exclusive access via `tcp_client_state()`, initialized once, never mutated concurrently.
unsafe impl Sync for TcpClientStateWrapper {}

static TCP_CLIENT_STATE: TcpClientStateWrapper = TcpClientStateWrapper {
    inner: UnsafeCell::new(None),
};

/// Returns the TCP client state.
///
/// # Safety
///
/// Call only after [`start_wifi`] has initialized [`TCP_CLIENT_STATE`].
pub unsafe fn tcp_client_state() -> &'static mut TcpClientState<1, 1500, 1500> {
    unsafe {
        let ptr = TCP_CLIENT_STATE.inner.get();
        (&mut *ptr).as_mut().unwrap()
    }
}

#[embassy_executor::task]
pub async fn wifi_start_task(spawner: Spawner, wifi: WIFI<'static>) {
    start_wifi(spawner, wifi).await;
}

async fn start_wifi(spawner: Spawner, wifi_peripheral: WIFI<'static>) {
    let access_point_station_config = Config::AccessPointStation(
        StationConfig::default(),
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

    let ap_ip = parse_ipv4(AP_IP_STR);
    let ap_gateway = parse_ipv4(AP_GATEWAY_STR);
    let ap_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(ap_ip, 24),
        gateway: Some(ap_gateway),
        dns_servers: Default::default(),
    });
    let sta_config = embassy_net::Config::dhcpv4(Default::default());

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // SAFETY: This is called once before any other task accesses TCP_CLIENT_STATE.
    unsafe {
        let ptr = TCP_CLIENT_STATE.inner.get();
        *ptr = Some(TcpClientState::<1, 1500, 1500>::new());
    }

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

    // Since only one client will ever connect, use a minimal DHCP server (2 slots — minimum required by leasehund).
    spawner.spawn(dhcp_server_task(ap_stack).unwrap());

    // Wait for the AP to become operational. Do not block on external Wi-Fi.
    ap_stack.wait_config_up().await;

    let ap = ap_ip.octets();
    info!("AP `{}` ready at http://{}.{}.{}.{}:8080/", WIFI_SSID, ap[0], ap[1], ap[2], ap[3]);

    spawner.spawn(wifi_connection_loop(ap_stack).unwrap());

    spawner.spawn(sta_http_poll_task(sta_stack, 60).unwrap());
}

/// DHCP server task for the AP — configured for a single client (minimum 2 slots required by leasehund).
#[embassy_executor::task]
async fn dhcp_server_task(stack: Stack<'static>) {
    let ap_ip = parse_ipv4(AP_IP_STR);
    let ap_subnet_mask = parse_ipv4(AP_SUBNET_MASK_STR);
    let ap_gateway = parse_ipv4(AP_GATEWAY_STR);
    let ap_dns_server = parse_ipv4(AP_DNS_SERVER_STR);
    let dhcp_client_ip = parse_ipv4(DHCP_CLIENT_IP_STR);
    let dhcp_pool_end = parse_ipv4(DHCP_POOL_END_STR);
    let mut server = DhcpServer::<2, 1>::new(
        ap_ip,          // Server IP (the AP itself)
        ap_subnet_mask, // Subnet mask
        ap_gateway,     // Router / gateway
        ap_dns_server,  // DNS server
        dhcp_client_ip, // First IP in the pool
        dhcp_pool_end,  // Last IP in the pool
    );
    let Err(e) = server.run(stack).await;
    info!("DHCP server error: {:?}", e);
}

/// Parse an IP address string at runtime (e.g., "192.168.2.1").
///
/// Returns `0.0.0.0` on malformed input (wrong number of octets or non-numeric characters).
fn parse_ipv4(s: &str) -> Ipv4Addr {
    let mut octets: [u8; 4] = [0; 4];
    let mut idx = 0usize;
    let mut val: u8 = 0;
    let mut valid = true;

    for &b in s.as_bytes() {
        if b == b'.' {
            if idx >= 4 {
                valid = false;
                break;
            }
            octets[idx] = val;
            idx += 1;
            val = 0;
        } else if b >= b'0' && b <= b'9' {
            val = val * 10 + (b - b'0');
        } else {
            valid = false;
            break;
        }
    }
    if valid && idx < 4 {
        octets[idx] = val;
    }
    if valid && idx == 3 {
        Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])
    } else {
        Ipv4Addr::new(0, 0, 0, 0)
    }
}
