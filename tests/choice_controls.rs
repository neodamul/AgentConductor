//! #409: behavioral checks against the real assembled webview, without a live agent.
//! GitHub #669: each script has a deadline; a stuck browser fails the test by name instead of holding the job until it is cancelled.
use std::{
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn drain(mut pipe: impl Read + Send + 'static) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = pipe.read_to_end(&mut bytes);
        bytes
    })
}

#[test]
fn anchored_choices_work_in_both_themes_and_reference_widths() {
    for script in [
        "runtime.cjs",
        "roles.cjs",
        "profile_fallback.cjs",
        "inspector_model.cjs",
    ] {
        let mut child = Command::new("node")
            .arg(format!("tests/choice_controls/{script}"))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Node starts the webview choice test");
        let (out, err) = (
            drain(child.stdout.take().unwrap()),
            drain(child.stderr.take().unwrap()),
        );
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break Some(status);
            }
            if started.elapsed() > Duration::from_secs(300) {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            std::thread::sleep(Duration::from_millis(200));
        };
        let (stdout, stderr) = (
            out.join().unwrap_or_default(),
            err.join().unwrap_or_default(),
        );
        let Some(status) = status else {
            panic!(
                "{script}: no result in 300 seconds, stopped\n{}\n{}",
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            );
        };
        if status.code() == Some(77) {
            eprintln!("{}", String::from_utf8_lossy(&stdout));
            continue;
        }
        assert!(
            status.success(),
            "{script}: {}\n{}",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr)
        );
    }
}
