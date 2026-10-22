//! Installed application icons for built-in providers (DD-40). Artwork is read from
//! the operator's own applications at runtime and never bundled.
use crate::protocol::Provider;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

mod fallback;
pub use fallback::fallback_icon;

const MAX_SIDE: u32 = 64;
const MAX_URL: usize = 64 * 1024;

/// The installed application icon for a built-in provider, if one can be read.
pub fn provider_icon(provider: Provider) -> Option<String> {
    if provider == Provider::Custom {
        return None;
    }
    static CACHE: OnceLock<Mutex<HashMap<Provider, Option<String>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(found) = cache.lock().ok()?.get(&provider) {
        return found.clone();
    }
    let icon = platform::icon(provider).or_else(|| fallback_icon(provider));
    cache.lock().ok()?.insert(provider, icon.clone());
    icon
}

/// Icons keyed by serialized provider name for the providers in use.
pub fn icon_map(providers: impl IntoIterator<Item = Provider>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for provider in providers {
        if let (Some(url), Ok(serde_json::Value::String(key))) =
            (provider_icon(provider), serde_json::to_value(provider))
        {
            map.insert(key, url.into());
        }
    }
    serde_json::Value::Object(map)
}

/// Encodes straight-alpha RGBA pixels as a PNG data URL.
pub fn png_data_url(width: u32, height: u32, rgba: &[u8]) -> Result<String, String> {
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return Err("icon dimensions must be 1..=64 pixels".into());
    }
    if rgba.len() != (width * height * 4) as usize {
        return Err("icon pixel buffer does not match its dimensions".into());
    }
    let mut raw = Vec::with_capacity(rgba.len() + height as usize);
    for row in rgba.chunks((width * 4) as usize) {
        raw.push(0); // filter: none
        raw.extend_from_slice(row);
    }
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit RGBA, no interlace
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    chunk(&mut png, b"IHDR", &header);
    chunk(&mut png, b"IDAT", &zlib_stored(&raw));
    chunk(&mut png, b"IEND", &[]);
    data_url(&png)
}

/// Wraps already-encoded PNG bytes, enforcing the snapshot size bound.
pub fn data_url(png: &[u8]) -> Result<String, String> {
    if !png.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("icon is not a PNG image".into());
    }
    let url = format!("data:image/png;base64,{}", base64(png));
    if url.len() > MAX_URL {
        return Err("icon exceeds 64 KiB".into());
    }
    Ok(url)
}

/// Decodes only small PNG data URLs produced for installed provider icons.
pub fn decode_data_url(url: &str) -> Result<Vec<u8>, String> {
    let encoded = url
        .strip_prefix("data:image/png;base64,")
        .ok_or("icon data URL must contain a PNG")?;
    if url.len() > MAX_URL || encoded.is_empty() || encoded.len() % 4 != 0 {
        return Err("icon data URL exceeds its size bound or has invalid base64 length".into());
    }
    let (groups, remainder) = encoded.as_bytes().as_chunks::<4>();
    if !remainder.is_empty() {
        return Err("invalid icon base64 length".into());
    }
    let mut png = Vec::with_capacity(encoded.len() * 3 / 4);
    let count = groups.len();
    for (index, group) in groups.iter().enumerate() {
        let last = index + 1 == count;
        let a = base64_digit(group[0]).ok_or("invalid icon base64")?;
        let b = base64_digit(group[1]).ok_or("invalid icon base64")?;
        let pad2 = group[2] == b'=';
        let pad1 = group[3] == b'=';
        let c = if pad2 {
            0
        } else {
            base64_digit(group[2]).ok_or("invalid icon base64")?
        };
        let d = if pad1 {
            0
        } else {
            base64_digit(group[3]).ok_or("invalid icon base64")?
        };
        if (!last && (pad2 || pad1))
            || (pad2 && !pad1)
            || (pad2 && b & 15 != 0)
            || (pad1 && !pad2 && c & 3 != 0)
        {
            return Err("invalid icon base64 padding".into());
        }
        let bits = (u32::from(a) << 18) | (u32::from(b) << 12) | (u32::from(c) << 6) | u32::from(d);
        png.push((bits >> 16) as u8);
        if !pad2 {
            png.push((bits >> 8) as u8);
        }
        if !pad1 {
            png.push(bits as u8);
        }
    }
    if !png.starts_with(b"\x89PNG\r\n\x1a\n") || png.len() < 24 {
        return Err("icon data URL is not a PNG".into());
    }
    let width = u32::from_be_bytes(png[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return Err("icon dimensions must be 1..=64 pixels".into());
    }
    Ok(png)
}

fn base64_digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = png.len();
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let crc = crc32(&png[start..]);
    png.extend_from_slice(&crc.to_be_bytes());
}

fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = data.chunks(65_535).collect();
    for (index, block) in blocks.iter().enumerate() {
        out.push(u8::from(index + 1 == blocks.len()));
        let length = block.len() as u16;
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&(!length).to_le_bytes());
        out.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in data {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    out.extend_from_slice(&((b << 16) | a).to_be_bytes());
    out
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for group in bytes.chunks(3) {
        let n = (u32::from(group[0]) << 16)
            | (u32::from(*group.get(1).unwrap_or(&0)) << 8)
            | u32::from(*group.get(2).unwrap_or(&0));
        for index in 0..4 {
            if index <= group.len() {
                out.push(TABLE[((n >> (18 - 6 * index)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(target_os = "macos")]
#[path = "icons/macos.rs"]
mod platform;
#[cfg(target_os = "macos")]
pub(crate) fn reduced_motion_preference() -> Option<bool> {
    platform::reduced_motion_preference()
}
#[cfg(windows)]
#[path = "icons/windows.rs"]
mod platform;
#[cfg(not(any(target_os = "macos", windows)))]
#[path = "icons/unsupported.rs"]
mod platform;
