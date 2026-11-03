use std::path::Path;

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
fn native_window_uses_the_embedded_application_icon() {
    let app = include_str!("../src/native_ui_probe/app.rs");
    assert!(app.contains(".with_icon(Arc::new(egui::IconData"));
    assert!(app.contains("include_bytes!(\"../ui/assets/ac-64.rgba\")"));
    assert!(!Path::new("tests/test_app_brand.py").exists());
}
