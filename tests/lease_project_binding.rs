use agentconductor::leases::Lease;
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
};

#[test]
fn project_binding_locks_are_scoped_per_canonical_folder() {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "agentconductor-binding-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let first = root.join("first");
    let second = root.join("second");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    let first_lease = Lease::project_binding(&first).unwrap();
    assert!(Lease::project_binding(&first).is_err());
    assert!(Lease::project_binding(&second).is_ok());
    drop(first_lease);
    assert!(Lease::project_binding(&first).is_ok());
    fs::remove_dir_all(root).unwrap();
}
