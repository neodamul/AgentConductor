use std::path::Path;

fn decode_base64(encoded: &str) -> Vec<u8> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bits = 0u32;
    let mut count = 0;
    let mut output = Vec::new();
    for byte in encoded.bytes().filter(|byte| *byte != b'=') {
        bits = (bits << 6) | alphabet.iter().position(|entry| *entry == byte).unwrap() as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            output.push((bits >> count) as u8);
        }
    }
    output
}

fn embedded_png(page: &str) -> Vec<u8> {
    let encoded = page
        .split_once("data:image/png;base64,")
        .unwrap()
        .1
        .split(['\'', '"'])
        .next()
        .unwrap();
    decode_base64(encoded)
}

#[test]
fn native_brand_assets_keep_valid_container_headers_and_dimensions() {
    let png = include_bytes!("../src/ui/assets/ac.png");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 1024);
    assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 1024);
    assert_eq!(
        &include_bytes!("../src/ui/assets/ac.ico")[..4],
        b"\0\0\x01\0"
    );
    let icns = include_bytes!("../src/ui/assets/ac.icns");
    assert_eq!(&icns[..4], b"icns");
    assert_eq!(
        u32::from_be_bytes(icns[4..8].try_into().unwrap()) as usize,
        icns.len()
    );
    assert_eq!(
        include_bytes!("../src/ui/assets/ac-64.rgba").len(),
        64 * 64 * 4
    );
}

#[test]
fn titlebars_embed_the_native_artwork_and_platform_shells_use_it() {
    let icon = include_bytes!("../src/ui/assets/ac-64.png");
    for page in [
        include_str!("../src/ui/startup.html"),
        include_str!("../src/ui/room.html"),
    ] {
        assert!(page.contains("class=\"brand-mark\""));
        assert_eq!(embedded_png(page), icon);
    }
    assert!(include_str!("../src/ui/native/cocoa/brand.rs").contains("setApplicationIconImage:"));
    assert!(include_str!("../src/ui/native/webview2.rs").contains("with_window_icon"));
    assert!(!Path::new("tests/test_app_brand.py").exists());
}
