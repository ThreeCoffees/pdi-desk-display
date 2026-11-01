use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[derive(Clone)]
pub struct WifiCredentials {
    pub ssid: heapless::String<32>,
    pub password: heapless::String<64>,
}

pub static WIFI_CONFIG_CHANNEL: Channel<CriticalSectionRawMutex, WifiCredentials, 1> =
    Channel::new();
