fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--version"] { println!("fixture 1"); return; }
    assert_eq!(args,["models"]);
    if let Some(path)=std::env::var_os("AC_METADATA_RELEASE") {
        let path=std::path::PathBuf::from(path);
        std::fs::write(path.with_extension("waiting"),b"waiting").unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        while !path.exists() { assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(10)); }
    }
    let count=std::env::var("AC_METADATA_COUNT").ok().and_then(|v|v.parse::<usize>().ok()).unwrap_or(1);
    if count==1 { println!("local/fixture-model"); } else { for i in 0..count { println!("local/model-{i}"); } }
}
