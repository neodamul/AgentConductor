//! Installed application icons for built-in providers (DD-40). Artwork is read from
//! the operator's own applications at runtime and never bundled.
use crate::protocol::Provider;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

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
    let icon = platform::icon(provider);
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
mod platform {
    use super::Provider;
    use std::ffi::{CString, c_char, c_void};
    use std::mem::transmute;

    type Id = *mut c_void;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }
    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {}
    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Id;
        fn objc_msgSend();
        fn objc_autoreleasePoolPush() -> *mut c_void;
        fn objc_autoreleasePoolPop(pool: *mut c_void);
    }
    fn class(name: &str) -> Id {
        let name = CString::new(name).unwrap();
        unsafe { objc_getClass(name.as_ptr()) }
    }
    fn sel(name: &str) -> Id {
        let name = CString::new(name).unwrap();
        unsafe { sel_registerName(name.as_ptr()) }
    }
    fn send0(object: Id, name: &str) -> Id {
        unsafe {
            transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id) -> Id>(objc_msgSend)(
                object,
                sel(name),
            )
        }
    }
    fn send1(object: Id, name: &str, arg: Id) -> Id {
        unsafe {
            transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id, Id) -> Id>(objc_msgSend)(
                object,
                sel(name),
                arg,
            )
        }
    }
    fn string(text: &str) -> Id {
        let text = CString::new(text).unwrap();
        unsafe {
            transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id, *const c_char) -> Id>(
                objc_msgSend,
            )(
                class("NSString"),
                sel("stringWithUTF8String:"),
                text.as_ptr(),
            )
        }
    }

    pub fn icon(provider: Provider) -> Option<String> {
        let bundle = match provider {
            Provider::Claude => "com.anthropic.claudefordesktop",
            Provider::Codex => "com.openai.codex",
            Provider::OpenCode => "ai.opencode.desktop",
            Provider::Custom => return None,
        };
        let pool = unsafe { objc_autoreleasePoolPush() };
        let result = render(bundle);
        unsafe { objc_autoreleasePoolPop(pool) };
        result
    }

    fn render(bundle: &str) -> Option<String> {
        let workspace = send0(class("NSWorkspace"), "sharedWorkspace");
        let url = send1(
            workspace,
            "URLForApplicationWithBundleIdentifier:",
            string(bundle),
        );
        if url.is_null() {
            return None;
        }
        let image = send1(workspace, "iconForFile:", send0(url, "path"));
        if image.is_null() {
            return None;
        }
        let side = 64isize;
        let rep = unsafe {
            transmute::<
                unsafe extern "C" fn(),
                extern "C" fn(
                    Id,
                    Id,
                    Id,
                    isize,
                    isize,
                    isize,
                    isize,
                    bool,
                    bool,
                    Id,
                    isize,
                    isize,
                ) -> Id,
            >(objc_msgSend)(
                send0(class("NSBitmapImageRep"), "alloc"),
                sel(
                    "initWithBitmapDataPlanes:pixelsWide:pixelsHigh:bitsPerSample:samplesPerPixel:hasAlpha:isPlanar:colorSpaceName:bytesPerRow:bitsPerPixel:",
                ),
                std::ptr::null_mut(),
                side,
                side,
                8,
                4,
                true,
                false,
                string("NSDeviceRGBColorSpace"),
                side * 4,
                32,
            )
        };
        if rep.is_null() {
            return None;
        }
        let context = send1(
            class("NSGraphicsContext"),
            "graphicsContextWithBitmapImageRep:",
            rep,
        );
        let graphics = class("NSGraphicsContext");
        send0(graphics, "saveGraphicsState");
        send1(graphics, "setCurrentContext:", context);
        let target = Rect {
            x: 0.0,
            y: 0.0,
            width: side as f64,
            height: side as f64,
        };
        let zero = Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
        unsafe {
            transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id, Rect, Rect, usize, f64)>(
                objc_msgSend,
            )(
                image,
                sel("drawInRect:fromRect:operation:fraction:"),
                target,
                zero,
                2, // NSCompositingOperationSourceOver
                1.0,
            );
        }
        send0(context, "flushGraphics");
        send0(graphics, "restoreGraphicsState");
        // NSBitmapImageFileTypePNG = 4
        let data = unsafe {
            transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id, usize, Id) -> Id>(
                objc_msgSend,
            )(
                rep,
                sel("representationUsingType:properties:"),
                4,
                send0(class("NSDictionary"), "dictionary"),
            )
        };
        let result = if data.is_null() {
            None
        } else {
            let length = send0(data, "length") as usize;
            let bytes = send0(data, "bytes") as *const u8;
            (!bytes.is_null())
                .then(|| unsafe { std::slice::from_raw_parts(bytes, length) }.to_vec())
                .and_then(|png| super::data_url(&png).ok())
        };
        send0(rep, "release");
        result
    }
}

#[cfg(windows)]
mod platform {
    use super::Provider;
    use std::path::PathBuf;
    use windows_sys::Win32::{
        Graphics::Gdi::{
            BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
            DeleteObject, GetDIBits,
        },
        System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx},
        UI::{
            Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW},
            WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO},
        },
    };

    /// Per-user desktop application locations (unverified until native acceptance).
    fn candidates(provider: Provider) -> Vec<PathBuf> {
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        let Some(local) = local else { return vec![] };
        match provider {
            Provider::Claude => vec![local.join("AnthropicClaude").join("claude.exe")],
            Provider::Codex => vec![
                local.join("Programs").join("ChatGPT").join("ChatGPT.exe"),
                local.join("Programs").join("Codex").join("Codex.exe"),
            ],
            Provider::OpenCode => vec![
                local.join("Programs").join("OpenCode").join("OpenCode.exe"),
                local.join("Programs").join("opencode").join("OpenCode.exe"),
            ],
            Provider::Custom => vec![],
        }
    }

    /// Start Menu shortcuts created by classic installers, directly or one folder deep.
    fn shortcuts(provider: Provider) -> Vec<PathBuf> {
        let names: &[&str] = match provider {
            Provider::Claude => &["claude.lnk"],
            Provider::Codex => &["chatgpt.lnk", "codex.lnk"],
            Provider::OpenCode => &["opencode.lnk"],
            Provider::Custom => &[],
        };
        let roots = ["APPDATA", "ProgramData"].into_iter().filter_map(|key| {
            std::env::var_os(key)
                .map(|base| PathBuf::from(base).join(r"Microsoft\Windows\Start Menu\Programs"))
        });
        let mut found = vec![];
        for root in roots {
            let entries = std::fs::read_dir(&root).into_iter().flatten().flatten();
            for entry in entries {
                let path = entry.path();
                let nested: Vec<PathBuf> = if path.is_dir() {
                    std::fs::read_dir(&path)
                        .into_iter()
                        .flatten()
                        .flatten()
                        .map(|e| e.path())
                        .collect()
                } else {
                    vec![path]
                };
                found.extend(nested.into_iter().filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| names.contains(&n.to_ascii_lowercase().as_str()))
                }));
            }
        }
        found
    }

    pub fn icon(provider: Provider) -> Option<String> {
        candidates(provider)
            .into_iter()
            .chain(shortcuts(provider))
            .filter(|path| path.is_file())
            .find_map(|path| extract(&path))
    }

    fn extract(path: &std::path::Path) -> Option<String> {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // Shell icon lookup resolves shortcut targets and requires COM on this thread.
        unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
        let mut info: SHFILEINFOW = unsafe { std::mem::zeroed() };
        let found = unsafe {
            SHGetFileInfoW(
                wide.as_ptr(),
                0,
                &mut info,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            )
        };
        let large: HICON = info.hIcon;
        if found == 0 || large.is_null() {
            return None;
        }
        let pixels = pixels(large);
        unsafe { DestroyIcon(large) };
        let (side, rgba) = pixels?;
        super::png_data_url(side, side, &rgba).ok()
    }

    fn pixels(icon: HICON) -> Option<(u32, Vec<u8>)> {
        let mut info: ICONINFO = unsafe { std::mem::zeroed() };
        if unsafe { GetIconInfo(icon, &mut info) } == 0 {
            return None;
        }
        let side = 32u32;
        let mut header: BITMAPINFO = unsafe { std::mem::zeroed() };
        header.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: side as i32,
            biHeight: -(side as i32), // top-down rows
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..unsafe { std::mem::zeroed() }
        };
        let mut bgra = vec![0u8; (side * side * 4) as usize];
        let dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
        let rows = unsafe {
            GetDIBits(
                dc,
                info.hbmColor,
                0,
                side,
                bgra.as_mut_ptr().cast(),
                &mut header,
                DIB_RGB_COLORS,
            )
        };
        unsafe {
            DeleteDC(dc);
            DeleteObject(info.hbmColor);
            DeleteObject(info.hbmMask);
        }
        if rows == 0 {
            return None;
        }
        for pixel in bgra.chunks_mut(4) {
            pixel.swap(0, 2);
        }
        Some((side, bgra))
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod platform {
    pub fn icon(_: super::Provider) -> Option<String> {
        None
    }
}
