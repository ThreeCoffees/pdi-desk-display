use embassy_net::{Runner, Stack};
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{Interface, WifiController};

use super::{connection_logic, http, rss};

#[embassy_executor::task]
pub async fn wifi_connection_loop(ap_stack: Stack<'static>) {
    http::serve(ap_stack).await;
}

#[embassy_executor::task]
pub async fn sta_http_poll_task(sta_stack: Stack<'static>, interval_secs: u64) {
    loop {
        rss::fetch_rss_feeds(sta_stack).await;
        Timer::after(Duration::from_secs(interval_secs)).await;
    }
}

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    connection_logic::run(&mut controller).await;
}

#[embassy_executor::task(pool_size = 2)]
pub async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await;
}
