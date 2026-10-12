#![cfg(windows)]
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
#[test]
fn windows_picker_owner_script_compiles_and_resolves_a_native_handle() {
    let output = agentconductor::subprocess::command("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-STA",
            "-Command",
            &picker::windows_owner_script("[Console]::Out.Write($o.Handle.ToInt64())"),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<isize>()
            .is_ok()
    );
}
