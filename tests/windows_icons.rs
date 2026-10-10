#![cfg(windows)]
//! DD-40 / QAS-27 on Windows: installed-path discovery, executable icon extraction and
//! PNG encoding through the real Win32 APIs. The application is simulated by placing a
//! system executable (which carries an icon resource) at the Claude per-user path.
use agentconductor::{icons, protocol::Provider};

#[test]
fn per_user_claude_install_path_yields_a_bounded_png_icon() {
    let local = std::env::temp_dir().join(format!("ac-win-icons-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&local);
    let app = local.join("AnthropicClaude");
    std::fs::create_dir_all(&app).unwrap();
    let system = std::env::var_os("SystemRoot").expect("SystemRoot");
    std::fs::copy(
        std::path::Path::new(&system)
            .join("System32")
            .join("notepad.exe"),
        app.join("claude.exe"),
    )
    .unwrap();
    // Single-threaded use before any icon lookup in this test binary.
    unsafe { std::env::set_var("LOCALAPPDATA", &local) };
    let url = icons::provider_icon(Provider::Claude).expect("icon from per-user install path");
    assert!(url.starts_with("data:image/png;base64,iVBORw0KGgo"));
    assert!(url.len() <= 64 * 1024);
    assert!(icons::provider_icon(Provider::Custom).is_none());
    let _ = std::fs::remove_dir_all(local);
}

#[test]
fn start_menu_shortcut_resolves_an_application_icon() {
    let roaming = std::env::temp_dir().join(format!("ac-win-menu-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&roaming);
    let menu = roaming.join(r"Microsoft\Windows\Start Menu\Programs\OpenCode");
    std::fs::create_dir_all(&menu).unwrap();
    let target = std::path::Path::new(&std::env::var_os("SystemRoot").unwrap())
        .join(r"System32\notepad.exe");
    let link = menu.join("OpenCode.lnk");
    let script = format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.Save()",
        link.display(),
        target.display()
    );
    assert!(
        std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .unwrap()
            .success()
    );
    // Single-threaded use before the OpenCode icon is looked up in this test binary.
    unsafe { std::env::set_var("APPDATA", &roaming) };
    let url = icons::provider_icon(Provider::OpenCode).expect("icon from Start Menu shortcut");
    assert!(url.starts_with("data:image/png;base64,iVBORw0KGgo"));
    let _ = std::fs::remove_dir_all(roaming);
}
