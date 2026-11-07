use defmt::error;
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_time::Duration;
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
            handlers::handle_wifi_get(socket).await?;
        }

        ("GET", "/rss") => {
            handlers::handle_rss_get(socket).await?;
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

pub async fn serve(ap_stack: Stack<'static>) {
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
