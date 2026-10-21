#![cfg(windows)]
use agentconductor::git;
#[test]
fn release_executable_uses_the_gui_subsystem() {
    let bytes = std::fs::read(env!("CARGO_BIN_EXE_agentconductor")).unwrap();
    let pe = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
    let subsystem = u16::from_le_bytes(bytes[pe + 24 + 68..pe + 24 + 70].try_into().unwrap());
    assert_eq!(subsystem, 2, "IMAGE_SUBSYSTEM_WINDOWS_GUI");
}
#[test]
fn background_console_program_preserves_pipes_without_a_console() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let source = root.join("console.rs");
    std::fs::write(&source, r#"#[link(name="kernel32")] unsafe extern "system" {fn GetConsoleWindow()->isize;} fn main(){assert_eq!(unsafe{GetConsoleWindow()},0); println!("stdout retained"); eprintln!("stderr retained");}"#).unwrap();
    let exe = root.join("console fixture.exe");
    assert!(
        std::process::Command::new("rustc")
            .arg(&source)
            .arg("-o")
            .arg(&exe)
            .status()
            .unwrap()
            .success()
    );
    let output = agentconductor::subprocess::command(&exe).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "stdout retained"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim(),
        "stderr retained"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[allow(dead_code)]
#[path = "../src/ui/picker.rs"]
mod picker;
// picker.rs reports errors through the app's message catalog (crate::ui::messages).
mod ui {
    pub use agentconductor::ui::{language, messages};
}
#[test]
fn native_windows_picker_returns_non_ascii_and_spaced_paths_without_a_code_page_roundtrip() {
    let raw: Vec<u16> = r"C:\사다리 프로젝트\🦀".encode_utf16().collect();
    assert_eq!(
        picker::windows::path_from_utf16(&raw).unwrap(),
        Some(std::path::PathBuf::from(r"C:\사다리 프로젝트\🦀"))
    );
    assert!(picker::windows::path_from_utf16(&[0xd800]).is_err());
}
