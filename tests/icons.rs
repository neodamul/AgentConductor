//! DD-40 / QAS-27: bounded PNG data URLs and built-in-only icons.
use agentconductor::{icons, protocol::Provider};

fn decode(data: &str) -> Vec<u8> {
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = vec![];
    for byte in data.bytes().filter(|b| *b != b'=') {
        bits = (bits << 6) | table.iter().position(|c| *c == byte).unwrap() as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
        }
    }
    out
}

fn crc(bytes: &[u8]) -> u32 {
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

#[test]
fn rgba_pixels_become_a_valid_bounded_png_data_url() {
    let rgba: Vec<u8> = (0..4 * 4 * 4).map(|i| i as u8).collect();
    let url = icons::png_data_url(4, 4, &rgba).unwrap();
    let encoded = url.strip_prefix("data:image/png;base64,").unwrap();
    let png = decode(encoded);
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    // Every chunk carries a correct CRC over its type and data.
    let mut at = 8;
    let mut kinds = vec![];
    while at < png.len() {
        let length = u32::from_be_bytes(png[at..at + 4].try_into().unwrap()) as usize;
        let body = &png[at + 4..at + 8 + length];
        let stored = u32::from_be_bytes(png[at + 8 + length..at + 12 + length].try_into().unwrap());
        assert_eq!(crc(body), stored);
        kinds.push(String::from_utf8(body[..4].to_vec()).unwrap());
        at += 12 + length;
    }
    assert_eq!(kinds, ["IHDR", "IDAT", "IEND"]);
    assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 4);
    assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 4);
}

#[test]
fn malformed_or_oversized_pixels_are_refused() {
    assert!(icons::png_data_url(4, 4, &[0; 3]).is_err());
    assert!(icons::png_data_url(0, 4, &[]).is_err());
    assert!(icons::png_data_url(65, 1, &vec![0; 65 * 4]).is_err());
}

#[test]
fn built_in_agents_use_the_pinned_brand_marks() {
    use sha2::{Digest, Sha256};
    let source = include_str!("../third_party/lobe-icons/SOURCE.md");
    for (provider, file) in [
        (Provider::Claude, "claude-code-color.svg"),
        (Provider::Codex, "codex-color.svg"),
        (Provider::OpenCode, "opencode.svg"),
    ] {
        let url = icons::provider_icon(provider).unwrap();
        let encoded = url.strip_prefix("data:image/svg+xml;base64,").unwrap();
        let svg = decode(encoded);
        assert_eq!(
            svg,
            std::fs::read(format!("third_party/lobe-icons/{file}")).unwrap()
        );
        assert!(url.len() <= 64 * 1024);
        // The vendored bytes are the pinned upstream files, so the notice's hash is checked.
        let hash: String = Sha256::digest(&svg)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(
            source.contains(&format!("`{file}` | `packages/")) && source.contains(&hash),
            "{file}"
        );
    }
    assert!(icons::brand_icon(Provider::Custom).is_none());
}

#[test]
fn custom_agents_never_receive_an_application_icon() {
    assert!(icons::provider_icon(Provider::Custom).is_none());
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires the OpenCode desktop application"]
fn installed_macos_applications_provide_their_icons() {
    let provider = Provider::OpenCode;
    let url = icons::provider_icon(provider).expect("installed application icon");
    assert!(url.starts_with("data:image/png;base64,"));
    assert!(url.len() <= 64 * 1024);
    if let Some(dir) = std::env::var_os("AC_ICON_EVIDENCE") {
        let png = decode(url.strip_prefix("data:image/png;base64,").unwrap());
        std::fs::write(
            std::path::Path::new(&dir).join(format!("{provider:?}.png")),
            png,
        )
        .unwrap();
    }
}

#[test]
fn pages_render_application_icons_only_for_built_in_agents() {
    for script in [
        include_str!("../src/ui/startup.js"),
        include_str!("../src/ui/room.js"),
    ] {
        assert!(script.contains("function avatar("));
        assert!(script.contains("app-icon"));
        assert!(script.contains("!=='custom'"));
    }
}

#[test]
fn built_in_badges_exist_without_desktop_applications() {
    let badges: Vec<_> = [Provider::Claude, Provider::Codex, Provider::OpenCode]
        .into_iter()
        .map(|p| icons::fallback_icon(p).unwrap())
        .collect();
    for b in &badges {
        assert!(b.starts_with("data:image/png;base64,"));
        assert!(b.len() < 8192);
    }
    assert!(badges[0] != badges[1] && badges[1] != badges[2]);
    assert!(icons::fallback_icon(Provider::Custom).is_none());
}
