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

pub async fn read_request<S>(socket: &mut S) -> Result<Option<HttpRequest>, S::Error>
where
    S: AsyncRead,
{
    let mut buffer = Vec::<u8, MAX_REQUEST>::new();

    let Some((header_end, content_length)) = read_headers(socket, &mut buffer).await? else {
        return Ok(None);
    };

    let total_length = header_end + content_length;

    if !read_body(socket, &mut buffer, total_length).await? {
        return Ok(None);
    }

    let Some((method_range, path_range)) = parse_request_line(&buffer, header_end) else {
        return Ok(None);
    };

    Ok(Some(HttpRequest {
        buffer,
        header_range: 0..header_end - 4,
        body_range: header_end..total_length,
        method_range,
        path_range,
    }))
}

async fn read_headers<S>(
    socket: &mut S,
    buffer: &mut Vec<u8, MAX_REQUEST>,
) -> Result<Option<(usize, usize)>, S::Error>
where
    S: AsyncRead,
{
    loop {
        if buffer.len() == MAX_REQUEST {
            return Ok(None);
        }

        let remaining = MAX_REQUEST - buffer.len();
        let mut temp = [0u8; MAX_REQUEST];

        let n = socket.read(&mut temp[..remaining]).await?;

        if n == 0 || buffer.extend_from_slice(&temp[..n]).is_err() {
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

        let Ok(headers) = str::from_utf8(&buffer[..end]) else {
            return Ok(None);
        };

        let Some(content_length) = parse_content_length(headers) else {
            return Ok(None);
        };

        if header_end + content_length > MAX_REQUEST {
            return Ok(None);
        }

        return Ok(Some((header_end, content_length)));
    }
}

fn parse_content_length(headers: &str) -> Option<usize> {
    let mut content_length = None;

    for line in headers.split("\r\n").skip(1) {
        let (name, value) = line.split_once(':')?;

        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return None;
            }

            content_length = Some(value.trim().parse::<usize>().ok()?);
        }

        if name.eq_ignore_ascii_case("transfer-encoding") {
            return None;
        }
    }

    let content_length = content_length.unwrap_or(0);

    if content_length > MAX_BODY {
        return None;
    }

    Some(content_length)
}

async fn read_body<S>(
    socket: &mut S,
    buffer: &mut Vec<u8, MAX_REQUEST>,
    total_length: usize,
) -> Result<bool, S::Error>
where
    S: AsyncRead,
{
    while buffer.len() < total_length {
        let remaining = total_length - buffer.len();
        let mut temp = [0u8; MAX_BODY];

        let n = socket.read(&mut temp[..remaining]).await?;

        if n == 0 || buffer.extend_from_slice(&temp[..n]).is_err() {
            return Ok(false);
        }
    }

    Ok(true)
}

fn parse_request_line(
    buffer: &[u8],
    header_end: usize,
) -> Option<(core::ops::Range<usize>, core::ops::Range<usize>)> {
    let request_line_end = find_bytes(&buffer[..header_end], b"\r\n")?;

    let request_line = str::from_utf8(&buffer[..request_line_end]).ok()?;
    let mut parts = request_line.split_ascii_whitespace();

    let method = parts.next()?;
    let path = parts.next()?;
    let version = parts.next()?;

    if parts.next().is_some()
        || !(version == "HTTP/1.0" || version == "HTTP/1.1")
        || !path.starts_with('/')
    {
        return None;
    }

    let method_start = method.as_ptr() as usize - buffer.as_ptr() as usize;
    let path_start = path.as_ptr() as usize - buffer.as_ptr() as usize;

    Some((
        method_start..method_start + method.len(),
        path_start..path_start + path.len(),
    ))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
