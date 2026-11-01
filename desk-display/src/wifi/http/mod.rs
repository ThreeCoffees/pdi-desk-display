use embedded_io_async::{Read as AsyncRead, Write as AsyncWrite};

mod form;
mod handlers;
mod request;
mod response;

use request::read_request;
use response::send_response;

const CONFIG_HTML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/web/config.html"));

pub async fn handle_http_connection<S>(socket: &mut S) -> Result<(), S::Error>
where
    S: AsyncRead + AsyncWrite,
{
    let Some(request) = read_request(socket).await? else {
        return send_response(
            socket,
            "400 Bad Request",
            "text/plain",
            "Invalid or incomplete request",
        )
        .await;
    };

    match (request.method(), request.path()) {
        ("GET", "/") => {
            send_response(socket, "200 OK", "text/html; charset=utf-8", CONFIG_HTML).await?;
        }

        ("GET", "/wifi") => {
            send_response(socket, "200 OK", "text/plain; charset=utf-8", "wifi ssid").await?;
        }

        ("GET", "/rss") => {
            send_response(socket, "200 OK", "text/plain; charset=utf-8", "rss feeds").await?;
        }

        ("POST", "/save-wifi") => {
            handlers::handle_wifi_form(socket, request.headers(), request.body()).await?;
        }

        ("POST", "/save-rss") => {
            handlers::handle_rss_form(socket, request.headers(), request.body()).await?;
        }

        _ => {
            send_response(socket, "404 Not Found", "text/plain", "Not found").await?;
        }
    }

    Ok(())
}
