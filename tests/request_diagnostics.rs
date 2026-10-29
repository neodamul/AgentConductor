#[test]
fn native_request_diagnostics_keep_the_bounded_chat_contract() {
    let source = include_str!("../src/ui/diagnostics.rs");
    assert!(source.contains("start kind={kind} bytes={size}"));
    assert!(source.contains("done kind={kind} ms={} ok={}{reason}"));
    assert!(source.contains("const LIMIT: u64 = 256 * 1024"));
    assert!(
        !source.contains("turns"),
        "diagnostics must remain content-free"
    );
}
