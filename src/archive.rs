//! DD-64: deterministic store-only ZIP for slug trees, implemented in-house (no new dependency).
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

const MAX_TOTAL: usize = 64 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;
/// 1980-01-01 00:00 in DOS format keeps archives byte-identical across exports.
const DOS_DATE: u16 = (1 << 5) | 1;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn entries(root: &Path) -> Result<Vec<(String, Option<Vec<u8>>)>, String> {
    let mut out = vec![];
    let mut stack = vec![PathBuf::new()];
    while let Some(relative) = stack.pop() {
        let mut children: Vec<_> = fs::read_dir(root.join(&relative))
            .map_err(|e| e.to_string())?
            .flatten()
            .collect();
        children.sort_by_key(|e| e.file_name());
        for child in children.into_iter().rev() {
            let path = relative.join(child.file_name());
            let name = path
                .to_str()
                .ok_or("non-UTF-8 path in slug tree")?
                .replace('\\', "/");
            let kind = child.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() {
                out.push((format!("{name}/"), None));
                stack.push(path);
            } else if kind.is_file() {
                out.push((
                    name,
                    Some(fs::read(child.path()).map_err(|e| e.to_string())?),
                ));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn write(entries: &[(String, Option<Vec<u8>>)]) -> Vec<u8> {
    let (mut body, mut central) = (vec![], vec![]);
    for (name, data) in entries {
        let data = data.as_deref().unwrap_or_default();
        let (crc, size, offset) = (crc32(data), data.len() as u32, body.len() as u32);
        let header = |sig: u32, central_dir: bool| {
            let mut h = sig.to_le_bytes().to_vec();
            if central_dir {
                h.extend(20u16.to_le_bytes());
            }
            for v in [20u16, 0x0800, 0, 0, DOS_DATE] {
                h.extend(v.to_le_bytes());
            }
            for v in [crc, size, size] {
                h.extend(v.to_le_bytes());
            }
            h.extend((name.len() as u16).to_le_bytes());
            h.extend(0u16.to_le_bytes());
            h
        };
        body.extend(header(0x0403_4b50, false));
        body.extend(name.as_bytes());
        body.extend(data);
        let mut c = header(0x0201_4b50, true);
        for v in [0u16, 0, 0] {
            c.extend(v.to_le_bytes());
        }
        c.extend(if name.ends_with('/') { 0x10u32 } else { 0 }.to_le_bytes());
        c.extend(offset.to_le_bytes());
        c.extend(name.as_bytes());
        central.extend(c);
    }
    let (count, offset, size) = (
        entries.len() as u16,
        body.len() as u32,
        central.len() as u32,
    );
    body.extend(central);
    body.extend(0x0605_4b50u32.to_le_bytes());
    for v in [0u16, 0, count, count] {
        body.extend(v.to_le_bytes());
    }
    body.extend(size.to_le_bytes());
    body.extend(offset.to_le_bytes());
    body.extend(0u16.to_le_bytes());
    body
}

pub fn pack_bytes(root: &Path) -> Result<Vec<u8>, String> {
    Ok(write(&entries(root)?))
}

pub fn pack(root: &Path, zip: &Path) -> Result<(), String> {
    fs::write(zip, pack_bytes(root)?).map_err(|e| e.to_string())
}

#[doc(hidden)]
/// Test helper: an archive with one arbitrary (possibly unsafe) entry name.
pub fn single_entry(name: &str, data: &[u8]) -> Vec<u8> {
    write(&[(name.to_string(), Some(data.to_vec()))])
}

fn u16_at(b: &[u8], at: usize) -> Result<u16, String> {
    b.get(at..at + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or_else(|| "truncated archive".into())
}
fn u32_at(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| "truncated archive".into())
}

fn safe_name(name: &str) -> Result<PathBuf, String> {
    let path = Path::new(name.trim_end_matches('/'));
    if name.is_empty()
        || name.contains('\\')
        || name.contains(':')
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("unsafe archive entry {name:?}"));
    }
    Ok(path.to_path_buf())
}

/// Validates every entry (names, method, CRC) before writing; extracts into a new `target`.
pub fn unpack(zip: &Path, target: &Path) -> Result<(), String> {
    if target.exists() {
        return Err(format!("{} already exists", target.display()));
    }
    let b = fs::read(zip).map_err(|e| format!("archive unreadable: {e}"))?;
    let end = (0..b.len().saturating_sub(21))
        .rev()
        .find(|&i| u32_at(&b, i).ok() == Some(0x0605_4b50))
        .ok_or("not a ZIP archive")?;
    let count = usize::from(u16_at(&b, end + 10)?);
    if count > MAX_ENTRIES {
        return Err("archive has too many entries".into());
    }
    let mut at = u32_at(&b, end + 16)? as usize;
    let (mut items, mut total) = (vec![], 0usize);
    for _ in 0..count {
        if u32_at(&b, at)? != 0x0201_4b50 {
            return Err("corrupt central directory".into());
        }
        let (method, crc) = (u16_at(&b, at + 10)?, u32_at(&b, at + 16)?);
        let size = u32_at(&b, at + 20)? as usize;
        let (name_len, extra, comment) = (
            usize::from(u16_at(&b, at + 28)?),
            usize::from(u16_at(&b, at + 30)?),
            usize::from(u16_at(&b, at + 32)?),
        );
        let local = u32_at(&b, at + 42)? as usize;
        let name = std::str::from_utf8(
            b.get(at + 46..at + 46 + name_len)
                .ok_or("truncated archive")?,
        )
        .map_err(|_| "non-UTF-8 entry name")?
        .to_string();
        at += 46 + name_len + extra + comment;
        if method != 0 {
            return Err(format!(
                "entry {name} is compressed; only stored entries are accepted"
            ));
        }
        let path = safe_name(&name)?;
        let skip = usize::from(u16_at(&b, local + 26)?) + usize::from(u16_at(&b, local + 28)?);
        if u32_at(&b, local)? != 0x0403_4b50 {
            return Err("corrupt local header".into());
        }
        let data = b
            .get(local + 30 + skip..local + 30 + skip + size)
            .ok_or("truncated entry")?;
        if crc32(data) != crc {
            return Err(format!("entry {name} failed its CRC check"));
        }
        total += size;
        if total > MAX_TOTAL {
            return Err("archive exceeds 64 MiB".into());
        }
        items.push((path, name.ends_with('/'), data.to_vec()));
    }
    let staging = target.with_extension(format!("unpacking-{}", std::process::id()));
    let _ = fs::remove_dir_all(&staging);
    let result = (|| {
        fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
        for (path, dir, data) in &items {
            let destination = staging.join(path);
            if *dir {
                fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
            } else {
                fs::create_dir_all(destination.parent().ok_or("invalid entry")?)
                    .map_err(|e| e.to_string())?;
                fs::write(&destination, data).map_err(|e| e.to_string())?;
            }
        }
        fs::rename(&staging, target).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}
