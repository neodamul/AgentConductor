//! #862 / DD-231: open a standard issue draft using the updater's repository and browser login.
use serde::Deserialize;
use serde_json::{Value, json};

pub mod logs;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    ReportBug {
        #[serde(default)]
        open: bool,
        // #1027: log ids from `bug_report_files` to zip into Downloads before the draft opens.
        #[serde(default)]
        logs: Vec<String>,
    },
    BugReportFiles {},
    // #921: the release page of the running version, on the same repository.
    ReleaseNotes {
        #[serde(default)]
        open: bool,
    },
}

fn encoded(text: &str) -> String {
    use std::fmt::Write;
    let mut result = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            result.push(char::from(byte));
        } else {
            write!(result, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    result
}

fn base(repository: &str) -> Result<(String, String), String> {
    let repository = crate::update_source::normalize_repository(repository)?;
    let repository = if repository.is_empty() {
        crate::update_source::DEFAULT_REPOSITORY.to_owned()
    } else {
        repository
    };
    let base = if repository.starts_with("https://") {
        repository.clone()
    } else {
        format!("https://github.com/{repository}")
    };
    Ok((repository, base))
}

pub fn release_notes(repository: &str) -> Result<Value, String> {
    let (repository, base) = base(repository)?;
    // A company build is tagged with its company-suffix.txt label (DD-165), so its notes live under that tag.
    let version = crate::update::current();
    let url = format!("{base}/releases/tag/v{version}");
    Ok(json!({"repository":repository,"version":version,"url":url,"opened":false}))
}

pub fn draft(repository: &str) -> Result<Value, String> {
    draft_with_logs(repository, None)
}

/// `zip` names the log archive the operator drags into the issue (#1027).
pub fn draft_with_logs(repository: &str, zip: Option<&str>) -> Result<Value, String> {
    let (repository, base) = base(repository)?;
    let logs = zip.map_or(String::new(), |name| {
        format!("## Logs\nDrag {name} from your Downloads folder into this issue.\n\n")
    });
    let body = format!(
        "## Summary\nDescribe the bug.\n\n## Steps to reproduce\n1. \n2. \n\n## Expected behavior\n\n## Actual behavior\n\n{logs}## Environment\n- AgentConductor: {}\n- OS: {}\n- Architecture: {}\n",
        crate::update::current(),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let url = format!("{base}/issues/new?body={}", encoded(&body));
    Ok(json!({"repository":repository,"body":body,"url":url,"opened":false}))
}

pub fn handle(request: Value) -> Result<Value, String> {
    let request = serde_json::from_value(request).map_err(|error| {
        crate::ui::messages::render(
            "native.bug_report_invalid",
            &[("error", &error.to_string())],
        )
    })?;
    let app = || {
        crate::desktop::workspace()?
            .parent()
            .map(std::path::Path::to_path_buf)
            .ok_or_else(|| "no application folder".to_owned())
    };
    let repository = crate::update_source::repository();
    let (mut report, open) = match request {
        Request::BugReportFiles {} => return Ok(logs::files(&app()?)),
        Request::ReportBug { open, logs } if !logs.is_empty() => {
            let app = app()?;
            let downloads = app
                .parent()
                .map_or_else(|| app.clone(), |home| home.join("Downloads"));
            std::fs::create_dir_all(&downloads).map_err(|e| e.to_string())?;
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            let name = format!("agentconductor-bug-report-{stamp}.zip");
            let zip = logs::write_zip(&app, &logs, &downloads, &name)?;
            if open {
                reveal(&zip)?;
            }
            let mut report = draft_with_logs(&repository, Some(&name))?;
            report["zip"] = zip.to_string_lossy().into_owned().into();
            (report, open)
        }
        Request::ReportBug { open, .. } => (draft(&repository)?, open),
        Request::ReleaseNotes { open } => (release_notes(&repository)?, open),
    };
    if open {
        open_browser(report["url"].as_str().expect("draft URL is a string"))?;
        report["opened"] = true.into();
    }
    Ok(report)
}

/// Shows the zip selected in Finder/Explorer so it can be dragged into the issue.
fn reveal(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = crate::subprocess::command("/usr/bin/open");
        command.arg("-R").arg(path);
        command
    };
    #[cfg(windows)]
    let mut command = {
        use std::os::windows::process::CommandExt;
        let mut command = crate::subprocess::command("explorer.exe");
        // explorer wants `/select,"path"` as one raw argument.
        command.raw_arg(format!("/select,\"{}\"", path.display()));
        command
    };
    #[cfg(not(any(windows, target_os = "macos")))]
    let mut command = {
        let mut command = crate::subprocess::command("xdg-open");
        command.arg(path.parent().unwrap_or(path));
        command
    };
    let mut child = command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(launch_error)?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

fn launch_error(error: impl std::fmt::Display) -> String {
    crate::ui::messages::render(
        "native.bug_report_browser_failed",
        &[("error", &error.to_string())],
    )
}

#[cfg(windows)]
pub(crate) fn open_browser(url: &str) -> Result<(), String> {
    use windows::Win32::System::Com::{
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
    };
    use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};
    // SAFETY: no reserved pointer is supplied; CoUninitialize below pairs this successful initialization.
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) }
        .ok()
        .map_err(launch_error)?;
    let verb: Vec<u16> = "open\0".encode_utf16().collect();
    let url: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();
    // SAFETY: all strings are terminated UTF-16 buffers alive for this call; optional pointers are null.
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            url.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    // SAFETY: this pairs the successful CoInitializeEx above on the same request thread.
    unsafe { CoUninitialize() };
    if result as isize > 32 {
        Ok(())
    } else {
        Err(launch_error(result as isize))
    }
}

#[cfg(not(windows))]
pub(crate) fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = crate::subprocess::command("/usr/bin/open");
    #[cfg(not(target_os = "macos"))]
    let mut command = crate::subprocess::command("xdg-open");
    let status = command
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(launch_error)?;
    if status.success() {
        Ok(())
    } else {
        Err(launch_error(status))
    }
}
