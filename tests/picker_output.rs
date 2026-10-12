//! R-82 / DD-90: a picked path reaches the form exactly, whatever the console encoding.
use agentconductor::ui::picker::picked;
use std::path::PathBuf;

#[test]
fn picker_output_drops_bom_line_ending_and_trailing_separator() {
    let bytes = "\u{feff}C:\\Users\\범준\\프로젝트\\\r\n"
        .as_bytes()
        .to_vec();
    assert_eq!(
        picked(bytes).unwrap(),
        Some(PathBuf::from("C:\\Users\\범준\\프로젝트"))
    );
}

#[test]
fn cancelled_picker_and_root_paths_are_preserved() {
    assert_eq!(picked(b"\r\n".to_vec()).unwrap(), None);
    assert_eq!(picked(b"/".to_vec()).unwrap(), Some(PathBuf::from("/")));
    assert!(
        picked(vec![0xC5, 0x41]).is_err(),
        "legacy code page bytes are refused"
    );
}
