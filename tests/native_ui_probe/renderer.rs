use super::*;

#[test]
fn native_renderer_matches_platform_backend() {
    #[cfg(windows)]
    assert!(matches!(renderer(), eframe::Renderer::Wgpu));
    #[cfg(not(windows))]
    assert!(matches!(renderer(), eframe::Renderer::Glow));
}
