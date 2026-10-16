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
    let legacy_alpha = bgra.chunks(4).all(|p| p[3] == 0);
    for pixel in bgra.chunks_mut(4) {
        if legacy_alpha {
            pixel[3] = 255;
        }
        pixel.swap(0, 2);
    }
    Some((side, bgra))
}
