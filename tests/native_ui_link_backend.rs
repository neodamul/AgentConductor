#[test]
fn desktop_eframe_enables_native_external_links() {
    let manifest = include_str!("../Cargo.toml");
    let dependency = manifest
        .lines()
        .find(|line| line.starts_with("eframe = { version = \"0.37.0\""))
        .expect("shared eframe dependency");
    assert!(
        dependency.contains("\"links\""),
        "the desktop backend drops OpenUrl commands unless eframe's links feature is enabled"
    );
}
