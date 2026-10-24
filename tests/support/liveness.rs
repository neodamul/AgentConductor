pub async fn assert_absent(pid: i32) {
    assert!(pid > 0, "invalid process probe PID");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        // SAFETY: signal 0 only probes for existence; `pid` is positive (asserted above) and no memory is passed.
        if unsafe { libc::kill(pid, 0) } != 0 {
            let error = std::io::Error::last_os_error();
            assert_eq!(
                error.raw_os_error(),
                Some(libc::ESRCH),
                "cannot prove process absence: {error}"
            );
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "descendant PID {pid} is still present"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
