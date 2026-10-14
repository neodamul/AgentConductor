use std::{
    fs,
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};
fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("Phased plan"));
    let marker = PathBuf::from(std::env::var("AC_NAV_TEST_RELEASE").unwrap());
    fs::write(
        marker.with_extension("ready"),
        std::process::id().to_string(),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut count = 0;
    while !marker.exists() {
        assert!(Instant::now() < deadline, "fixture was not released");
        count += 1;
        fs::write(marker.with_extension("progress"), count.to_string()).unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }
    println!(
        "{}",
        r#"{"type":"result","payload":{"slug":"background_plan","analysis":"Analysis","phases":[{"id":"01-core","name":"Core","tasks":[{"id":"a","agent":"mock","prompt":"one.txt","paths":["one.txt"],"depends_on":[],"test_command":["git","diff","--check"]}]}]}}"#
    );
}
