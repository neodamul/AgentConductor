use agentconductor::leases::Lease;

#[test]
#[ignore = "subprocess helper invoked by killed_owner_releases_kernel_lease"]
fn lease_child() {
    let path = std::env::var_os("AC_TEST_LEASE").unwrap();
    let _lease = Lease::acquire(std::path::Path::new(&path)).unwrap();
    std::fs::write(std::env::var_os("AC_TEST_READY").unwrap(), "ready").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(30));
}

#[test]
fn killed_owner_releases_kernel_lease() {
    let dir = std::env::temp_dir().join(format!("ac-lease-killed-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("coordinator.lock");
    let ready = dir.join("ready");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "lease_child", "--ignored"])
        .env("AC_TEST_LEASE", &path)
        .env("AC_TEST_READY", &ready)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !ready.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let is_ready = ready.exists();
    let held = Lease::acquire(&path).is_err();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(is_ready && held, "child did not hold the lease");
    let lease = Lease::acquire(&path).unwrap();
    drop(lease);
    assert!(path.exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn exclusive_lease_is_released_without_deleting_its_file() {
    let dir = std::env::temp_dir().join(format!("ac-lease-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("coordinator.lock");
    std::fs::write(&path, "stale owner record\n").unwrap();
    let lease = Lease::acquire(&path).unwrap();
    assert!(Lease::acquire(&path).is_err());
    drop(lease);
    assert!(path.exists());
    let lease = Lease::acquire(&path).unwrap();
    drop(lease);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn lock_symlinks_are_rejected() {
    use std::os::unix::fs::symlink;
    let dir = std::env::temp_dir().join(format!("ac-lease-symlink-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let target = dir.join("target");
    std::fs::write(&target, "untouched").unwrap();
    let path = dir.join("lock");
    symlink(&target, &path).unwrap();
    assert!(Lease::acquire(&path).is_err());
    assert_eq!(std::fs::read_to_string(target).unwrap(), "untouched");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn repository_leases_are_exclusive_per_canonical_repository() {
    let dir = std::env::temp_dir().join(format!("ac-lease-repo-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let lease = Lease::repository(&dir).unwrap();
    assert!(Lease::repository(&dir).is_err());
    drop(lease);
    drop(Lease::repository(&dir).unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}
