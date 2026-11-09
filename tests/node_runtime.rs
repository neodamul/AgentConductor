//! GitHub #273: every UI behavior test in `tests/*.cjs` runs in CI, not only the ones a Rust test
//! names; `start_feedback_runtime.cjs` had been failing unnoticed because nothing ran it.
#[test]
fn every_node_ui_test_passes() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut scripts: Vec<_> = std::fs::read_dir(std::path::Path::new(root).join("tests"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cjs"))
        .collect();
    scripts.sort();
    assert!(scripts.len() > 20, "the UI tests are found");
    // GitHub #651: the page tests are independent, so two run at a time instead of one after another.
    let next = std::sync::atomic::AtomicUsize::new(0);
    let failed = std::sync::Mutex::new(Vec::<String>::new());
    let run = |script: &std::path::PathBuf| -> Option<String> {
        {
            // A page test that hangs (a browser that never exits) must name itself instead of stalling the whole job.
            let mut child = std::process::Command::new("node")
                .current_dir(root)
                .arg(script)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("Node runs UI behavior tests");
            let (mut out, mut err) = (child.stdout.take().unwrap(), child.stderr.take().unwrap());
            let readers = [
                std::thread::spawn(move || {
                    let mut v = Vec::new();
                    let _ = std::io::Read::read_to_end(&mut out, &mut v);
                    v
                }),
                std::thread::spawn(move || {
                    let mut v = Vec::new();
                    let _ = std::io::Read::read_to_end(&mut err, &mut v);
                    v
                }),
            ];
            let started = std::time::Instant::now();
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break Some(status);
                }
                if started.elapsed() > std::time::Duration::from_secs(120) {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            };
            let [_, stderr] = readers.map(|reader| reader.join().unwrap_or_default());
            (!status.is_some_and(|status| status.success())).then(|| {
                format!(
                    "{}: {}\n{}",
                    script.display(),
                    if status.is_none() {
                        "hung for 120 seconds and was stopped"
                    } else {
                        "failed"
                    },
                    String::from_utf8_lossy(&stderr)
                )
            })
        }
    };
    std::thread::scope(|scope| {
        for _ in 0..2 {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(script) = scripts.get(index) else {
                        break;
                    };
                    if let Some(message) = run(script) {
                        failed.lock().unwrap().push(message);
                    }
                }
            });
        }
    });
    let mut failed = failed.into_inner().unwrap();
    failed.sort();
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
