use core::fmt::Write;
use embedded_io_async::Write as AsyncWrite;

pub async fn send_response<S>(
    socket: &mut S,
    status: &str,
    content_type: &str,
    body: &str,
) -> Result<(), S::Error>
where
    S: AsyncWrite,
{
    let mut header = heapless::String::<192>::new();

    let _ = write!(
        &mut header,
        "HTTP/1.1 {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Cache-Control: no-store\r\n\
         \r\n",
        status,
        content_type,
        body.len(),
    );

    socket.write_all(header.as_bytes()).await?;
    socket.write_all(body.as_bytes()).await?;
    socket.flush().await?;

    Ok(())
}

pub async fn send_redirect<S>(socket: &mut S) -> Result<(), S::Error>
where
    S: AsyncWrite,
{
    socket
        .write_all(
            b"HTTP/1.1 303 See Other\r\n\
          Location: /\r\n\
          Content-Length: 0\r\n\
          Cache-Control: no-store\r\n\
          Connection: close\r\n\
          \r\n",
        )
        .await?;

    socket.flush().await?;

    Ok(())
}

