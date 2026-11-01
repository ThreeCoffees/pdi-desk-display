use core::ops::Range;
use core::str;

use embedded_io_async::Read as AsyncRead;
use heapless::Vec;

pub const MAX_REQUEST: usize = 1536;
pub const MAX_HEADER: usize = 1024;
pub const MAX_BODY: usize = 512;

/// An HTTP request whose contents are stored in a fixed-size buffer.
pub struct HttpRequest {
    buffer: Vec<u8, MAX_REQUEST>,
    header_range: Range<usize>,
    body_range: Range<usize>,
    method_range: Range<usize>,
    path_range: Range<usize>,
}

impl HttpRequest {
    pub fn method(&self) -> &str {
        str::from_utf8(&self.buffer[self.method_range.clone()]).unwrap_or("")
    }

    pub fn path(&self) -> &str {
        str::from_utf8(&self.buffer[self.path_range.clone()]).unwrap_or("")
    }

    pub fn headers(&self) -> &str {
        str::from_utf8(&self.buffer[self.header_range.clone()]).unwrap_or("")
    }

    pub fn body(&self) -> &[u8] {
        &self.buffer[self.body_range.clone()]
    }
}

/// Read one complete HTTP request.
pub async fn read_request<S>(socket: &mut S) -> Result<Option<HttpRequest>, S::Error>
where
    S: AsyncRead,
{
    let mut buffer = Vec::<u8, MAX_REQUEST>::new();

    let (header_end, content_length) = loop {
        if buffer.len() == MAX_REQUEST {
            return Ok(None);
        }

        let old_len = buffer.len();

        // Reserve enough space for the next read.
        let remaining = MAX_REQUEST - old_len;
        let mut temp = [0u8; MAX_REQUEST];

        let n = socket.read(&mut temp[..remaining]).await?;

        if n == 0 {
            return Ok(None);
        }

        if buffer.extend_from_slice(&temp[..n]).is_err() {
            return Ok(None);
        }

        let Some(end) = find_bytes(buffer.as_slice(), b"\r\n\r\n") else {
            if buffer.len() > MAX_HEADER {
                return Ok(None);
            }
            continue;
        };

        let header_end = end + 4;

        if header_end > MAX_HEADER {
            return Ok(None);
        }

        let headers = match str::from_utf8(&buffer[..end]) {
            Ok(headers) => headers,
            Err(_) => return Ok(None),
        };

        let mut content_length = None;

        for line in headers.split("\r\n").skip(1) {
            let Some((name, value)) = line.split_once(':') else {
                return Ok(None);
            };

            if name.eq_ignore_ascii_case("content-length") {
                // Reject duplicate Content-Length headers.
                if content_length.is_some() {
                    return Ok(None);
                }

                let Ok(length) = value.trim().parse::<usize>() else {
                    return Ok(None);
                };

                content_length = Some(length);
            }

            if name.eq_ignore_ascii_case("transfer-encoding") {
                // Chunked request bodies are not supported.
                return Ok(None);
            }
        }

        let content_length = content_length.unwrap_or(0);

        if content_length > MAX_BODY || header_end + content_length > MAX_REQUEST {
            return Ok(None);
        }

        break (header_end, content_length);
    };

    let total_length = header_end + content_length;

    // Read the remainder of the declared body.
    while buffer.len() < total_length {
        let old_len = buffer.len();
        let remaining = total_length - old_len;

        let mut temp = [0u8; MAX_BODY];
        let n = socket.read(&mut temp[..remaining]).await?;

        if n == 0 {
            return Ok(None);
        }

        if buffer.extend_from_slice(&temp[..n]).is_err() {
            return Ok(None);
        }
    }

    // Parse request line.
    let request_line_end = find_bytes(&buffer[..header_end], b"\r\n").unwrap_or(header_end - 2);

    let (method_range, path_range) = {
        let request_line = match str::from_utf8(&buffer[..request_line_end]) {
            Ok(line) => line,
            Err(_) => return Ok(None),
        };
        let mut parts = request_line.split_ascii_whitespace();

        let Some(method) = parts.next() else {
            return Ok(None);
        };

        let Some(path) = parts.next() else {
            return Ok(None);
        };

        let Some(version) = parts.next() else {
            return Ok(None);
        };

        if parts.next().is_some()
            || !(version == "HTTP/1.0" || version == "HTTP/1.1")
            || !path.starts_with('/')
        {
            return Ok(None);
        }

        let method_start = method.as_ptr() as usize - buffer.as_ptr() as usize;

        let path_start = path.as_ptr() as usize - buffer.as_ptr() as usize;

        (
            method_start..method_start + method.len(),
            path_start..path_start + path.len(),
        )
    };

    let header_range = 0..header_end - 4;
    let body_range = header_end..total_length;

    Ok(Some(HttpRequest {
        buffer,
        header_range,
        body_range,
        method_range,
        path_range,
    }))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
