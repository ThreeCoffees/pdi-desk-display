use core::str;

use defmt::{debug, error, info};
use embassy_net::{
    Stack,
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use reqwless::{
    client::{HttpClient, TlsConfig, TlsVerify},
    request::Method,
};
use rss_parser::parser::RssParser;
use rss_parser::types::RssFeed;

use super::TCP_CLIENT_STATE;
use super::rss_config::get_rss_feeds;

pub async fn fetch_rss_feeds(sta_stack: Stack<'static>) {
    if !is_station_ready(&sta_stack) {
        debug!("Station is not connected; skipping RSS fetch");
        return;
    }

    let feeds = get_rss_feeds();

    if feeds.is_empty() {
        debug!("No RSS feeds configured");
        return;
    }

    let tcp_client = TcpClient::new(
        sta_stack,
        TCP_CLIENT_STATE.init(TcpClientState::<1, 1500, 1500>::new()),
    );

    let dns_client = DnsSocket::new(sta_stack);
    let mut rx_buffer = [0u8; 4096];

    for url in feeds.iter() {
        fetch_single_feed(url, &tcp_client, &dns_client, &mut rx_buffer).await;
    }
}

fn is_station_ready(sta_stack: &Stack<'static>) -> bool {
    sta_stack.is_link_up() && sta_stack.config_v4().is_some()
}

async fn fetch_single_feed(
    url: &str,
    tcp_client: &TcpClient<'_, 1, 1500, 1500>,
    dns_client: &DnsSocket<'_>,
    rx_buffer: &mut [u8],
) {
    info!("Fetching RSS feed: {}", url);

    let mut tls_rx_buffer = [0u8; 16640];
    let mut tls_tx_buffer = [0u8; 16640];

    let tls_seed = generate_tls_seed();

    let tls = TlsConfig::new(
        tls_seed,
        &mut tls_rx_buffer,
        &mut tls_tx_buffer,
        TlsVerify::None,
    );

    let mut client = HttpClient::new_with_tls(tcp_client, dns_client, tls);

    let mut builder = match client.request(Method::GET, url).await {
        Ok(builder) => builder,
        Err(_) => {
            error!("RSS DNS/connect error for {}", url);
            return;
        }
    };

    let response = match builder.send(rx_buffer).await {
        Ok(response) => response,
        Err(_) => {
            error!("RSS request failed for {}", url);
            return;
        }
    };

    info!("RSS request succeeded: {}", url);

    let mut parser = RssParser::new();
    let total_bytes = read_feed_body(url, response.body().reader(), &mut parser).await;

    info!("RSS feed {}: {} bytes received", url, total_bytes);

    match parser.finish() {
        Ok(feed) => {
            info!("Feed \"{}\": {} items", feed.title.as_str(), feed.items.len());
            for item in feed.items.iter() {
                info!("  - {}", item.title.as_str());
            }
        }
        Err(e) => {
            error!("RSS parse error: {}", e);
        }
    }
}

fn generate_tls_seed() -> u64 {
    let rng = esp_hal::rng::Rng::new();

    (rng.random() as u64) << 32 | rng.random() as u64
}

async fn read_feed_body<R>(url: &str, mut body: R, parser: &mut RssParser) -> usize
where
    R: embedded_io_async::Read,
{
    let mut chunk = [0u8; 256];
    let mut total_bytes = 0usize;

    loop {
        let n = match body.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => {
                error!("RSS read error for {}", url);
                break;
            }
        };

        total_bytes += n;

        if let Ok(text) = str::from_utf8(&chunk[..n]) {
            parser.push(text);
        }
    }

    total_bytes
}
