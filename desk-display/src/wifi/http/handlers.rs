
use defmt::info;
use embedded_io_async::Write as AsyncWrite;
use heapless::{String, Vec};

use crate::wifi::wifi_config::{WIFI_CONFIG, WIFI_CONFIG_CHANNEL, WifiCredentials};
use crate::wifi::http::form::is_urlencoded_form;
use crate::wifi::http::response::send_redirect;
use crate::wifi::http::{form::get_form_value, response::send_response};
use crate::wifi::rss_config::{get_rss_feeds, save_rss_feeds};

pub async fn handle_rss_get<S>(socket: &mut S) -> Result<(), S::Error>
where
    S: AsyncWrite,
{
    let rss_feeds = get_rss_feeds().to_string();

    send_response(
        socket,
        "200 OK",
        "text/plain; charset=utf-8",
        rss_feeds.as_str(),
    )
    .await
}

pub async fn handle_rss_form<S>(socket: &mut S, headers: &str, body: &[u8]) -> Result<(), S::Error>
where
    S: AsyncWrite,
{
    if !is_urlencoded_form(headers) {
        return send_response(
            socket,
            "415 Unsupported Media Type",
            "text/plain",
            "Expected URL-encoded form",
        )
        .await;
    }

    let mut rss_feeds_buffer = Vec::<u8, 128>::new();

    let Some(rss_feeds) = get_form_value(body, "rss-feeds", &mut rss_feeds_buffer) else {
        return send_response(
            socket,
            "400 Bad Request",
            "text/plain",
            "Missing or invalid RSS feeds",
        )
        .await;
    };

    if save_rss_feeds(rss_feeds) {
        send_redirect(socket).await
    } else {
        send_response(
            socket,
            "500 Internal Server Error",
            "text/plain",
            "Could not save RSS feeds",
        )
        .await
    }
}

pub async fn handle_wifi_get<S>(socket: &mut S) -> Result<(), S::Error>
where
    S: AsyncWrite,
{
    let ssid = WIFI_CONFIG.lock(|config| {
        config
            .borrow()
            .as_ref()
            .map(|credentials| credentials.ssid.clone())
    });

    match ssid {
        Some(ssid) => {
            send_response(socket, "200 OK", "text/plain; charset=utf-8", ssid.as_str()).await
        }
        None => send_response(socket, "200 OK", "text/plain; charset=utf-8", "").await,
    }
}

pub async fn handle_wifi_form<S>(socket: &mut S, headers: &str, body: &[u8]) -> Result<(), S::Error>
where
    S: AsyncWrite,
{
    if !is_urlencoded_form(headers) {
        return send_response(
            socket,
            "415 Unsupported Media Type",
            "text/plain",
            "Expected URL-encoded form",
        )
        .await;
    }

    let mut ssid_buf = Vec::<u8, 128>::new();
    let mut password_buf = Vec::<u8, 128>::new();

    let ssid = get_form_value(body, "ssid", &mut ssid_buf);
    let password = get_form_value(body, "password", &mut password_buf);

    match (ssid, password) {
        (Some(ssid), Some(password)) => {
            if save_credentials(ssid, password).await {
                send_redirect(socket).await?
            } else {
                send_response(
                    socket,
                    "400 Bad Request",
                    "text/plain",
                    "Invalid Wi-Fi credentials",
                )
                .await?
            }
        }
        _ => {
            send_response(
                socket,
                "400 Bad Request",
                "text/plain",
                "Missing or invalid Wi-Fi fields",
            )
            .await?;
        }
    }

    Ok(())
}

async fn save_credentials(ssid: &str, password: &str) -> bool {
    let mut ssid_string = String::<32>::new();
    let mut password_string = String::<64>::new();

    if ssid_string.push_str(ssid).is_err() || password_string.push_str(password).is_err() {
        return false;
    }

    let credentials = WifiCredentials {
        ssid: ssid_string,
        password: password_string,
    };

    WIFI_CONFIG.lock(|config| {
        config.borrow_mut().replace(credentials.clone());
    });
    WIFI_CONFIG_CHANNEL.send(credentials).await;

    info!("New wifi credentials set. SSID: {}", ssid);

    true
}
