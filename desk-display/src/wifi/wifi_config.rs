use core::cell::RefCell;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

#[derive(Clone)]
pub struct WifiCredentials {
    pub ssid: heapless::String<32>,
    pub password: heapless::String<64>,
}

pub static WIFI_CONFIG_CHANNEL: Channel<CriticalSectionRawMutex, WifiCredentials, 1> =
    Channel::new();

pub static WIFI_CONFIG: Mutex<CriticalSectionRawMutex, RefCell<Option<WifiCredentials>>> =
    Mutex::new(RefCell::new(None));
