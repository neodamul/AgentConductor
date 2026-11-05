//! Minimal HTTP/1.1 response framing for the local memory server: headers, bounded reads, chunked bodies.
use super::{BODY_MAX, WIRE_MAX};
use std::{
    io::Read,
    net::TcpStream,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub(super) enum Framing {
    Length(usize),
    Chunked,
}
pub(super) fn headers(bytes: &[u8]) -> Result<(u16, Framing), String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "invalid memory response headers")?;
    let mut lines = text.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    if !matches!(first.next(), Some("HTTP/1.1" | "HTTP/1.0")) {
        return Err("invalid memory response protocol".into());
    }
    let status = first
        .next()
        .ok_or("missing memory response status")?
        .parse::<u16>()
        .map_err(|_| "invalid memory response status")?;
    if !(100..=599).contains(&status) {
        return Err("invalid memory response status".into());
    }
    let mut length = None;
    let mut transfer = false;
    for line in lines {
        let (key, value) = line
            .split_once(':')
            .ok_or("malformed memory response header")?;
        let value = value.trim();
        if key.eq_ignore_ascii_case("content-length") {
            let n = value
                .parse::<usize>()
                .map_err(|_| "invalid memory content length")?;
            if n > BODY_MAX || length.replace(n).is_some() {
                return Err("oversized or duplicate memory content length".into());
            }
        } else if key.eq_ignore_ascii_case("transfer-encoding") {
            if transfer || !value.eq_ignore_ascii_case("chunked") {
                return Err("unsupported memory transfer encoding".into());
            }
            transfer = true;
        } else if key.eq_ignore_ascii_case("content-encoding")
            && !value.eq_ignore_ascii_case("identity")
        {
            return Err("unsupported memory content encoding".into());
        }
    }
    match (length, transfer) {
        (Some(n), false) => Ok((status, Framing::Length(n))),
        (None, true) => Ok((status, Framing::Chunked)),
        _ => Err("ambiguous or missing memory response framing".into()),
    }
}
pub(super) fn read_more(
    stream: &mut TcpStream,
    data: &mut Vec<u8>,
    buf: &mut [u8],
    started: Instant,
    deadline: Duration,
) -> Result<bool, String> {
    let left = deadline
        .checked_sub(started.elapsed())
        .filter(|d| !d.is_zero())
        .ok_or("memory response deadline exceeded")?;
    stream
        .set_read_timeout(Some(left))
        .map_err(|e| e.to_string())?;
    let n = stream
        .read(buf)
        .map_err(|e| format!("memory response unavailable: {e}"))?;
    if data.len() + n > WIRE_MAX {
        return Err("memory response exceeds wire limit".into());
    }
    data.extend_from_slice(&buf[..n]);
    Ok(n == 0)
}
pub(super) fn chunked(data: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let mut cursor = 0;
    let mut body = vec![];
    for _ in 0..1024 {
        let Some(offset) = data[cursor..].windows(2).position(|s| s == b"\r\n") else {
            return Ok(None);
        };
        if offset > 128 {
            return Err("memory chunk header exceeds limit".into());
        }
        let text = std::str::from_utf8(&data[cursor..cursor + offset])
            .map_err(|_| "invalid memory chunk header")?;
        let count = usize::from_str_radix(text.split(';').next().unwrap_or_default(), 16)
            .map_err(|_| "invalid memory chunk length")?;
        cursor += offset + 2;
        if count == 0 {
            if data.len() < cursor + 2 {
                return Ok(None);
            }
            if &data[cursor..cursor + 2] != b"\r\n" {
                return Err("memory chunk trailers unsupported".into());
            }
            return Ok(Some(body));
        }
        if count > BODY_MAX - body.len() {
            return Err("memory chunked body exceeds limit".into());
        }
        if data.len() < cursor + count + 2 {
            return Ok(None);
        }
        body.extend_from_slice(&data[cursor..cursor + count]);
        cursor += count;
        if &data[cursor..cursor + 2] != b"\r\n" {
            return Err("invalid memory chunk terminator".into());
        }
        cursor += 2;
    }
    Err("too many memory response chunks".into())
}
