#![cfg(windows)]
//! DD-40 / QAS-27 on Windows: installed-path discovery, executable icon extraction and
//! PNG encoding through the real Win32 APIs. The application is simulated by placing a
//! system executable (which carries an icon resource) at the Claude per-user path.
use agentconductor::{icons, protocol::Provider};

#[path = "support/env.rs"]
mod test_env;

#[test]
fn an_installed_claude_application_does_not_replace_the_brand_mark() {
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
    let mut env = test_env::lock();
    env.set("LOCALAPPDATA", &local);
    // GitHub #406: Claude Code keeps its pinned brand mark even when the application is installed.
    let url = icons::provider_icon(Provider::Claude).expect("bundled brand mark");
    assert!(url.starts_with("data:image/svg+xml;base64,"));
    assert!(url.len() <= 64 * 1024);
    assert!(icons::provider_icon(Provider::Custom).is_none());
    let _ = std::fs::remove_dir_all(local);
}

#[test]
fn an_installed_start_menu_shortcut_does_not_replace_the_opencode_brand_mark() {
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
    let mut env = test_env::lock();
    env.set("APPDATA", &roaming);
    // GitHub #439: OpenCode keeps its pinned brand mark even when a Start Menu shortcut exists.
    let url = icons::provider_icon(Provider::OpenCode).expect("bundled brand mark");
    assert!(url.starts_with("data:image/svg+xml;base64,"));
    let _ = std::fs::remove_dir_all(roaming);
}
