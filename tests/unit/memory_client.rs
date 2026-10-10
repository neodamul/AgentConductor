use super::*;
#[test]
fn existing_daemon_secret_precedence_is_bounded_and_never_created() {
    let path = std::env::temp_dir().join(format!(
        "ac-memory-auth-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&path).unwrap();
    assert_eq!(load_secret(&path).unwrap(), None);
    assert!(!path.join("secret").exists());
    std::fs::write(path.join("secret"), "stored-fixture-secret\n").unwrap();
    assert_eq!(
        load_secret(&path).unwrap().unwrap(),
        "stored-fixture-secret"
    );
    std::fs::write(
        path.join(".env"),
        "# ignored\nexport AGENTMEMORY_SECRET = 'configured-fixture-secret' # comment\n",
    )
    .unwrap();
    assert_eq!(
        load_secret(&path).unwrap().unwrap(),
        "configured-fixture-secret"
    );
    std::fs::write(path.join(".env"), "AGENTMEMORY_SECRET=${UNRESOLVED}\n").unwrap();
    assert_eq!(
        load_secret(&path).unwrap().unwrap(),
        "stored-fixture-secret"
    );
    std::fs::write(path.join(".env"), vec![b'x'; 65537]).unwrap();
    assert!(load_secret(&path).is_err());
    std::fs::remove_dir_all(path).unwrap();
}
