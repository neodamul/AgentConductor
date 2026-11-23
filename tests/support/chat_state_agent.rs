use std::{
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};

fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    std::fs::write(std::env::var_os("AC_CHAT_CAPTURE").unwrap(), prompt).unwrap();

    // Keep the job active until the parent has checked the duplicate request.
    let release = PathBuf::from(std::env::var_os("AC_CHAT_RELEASE").unwrap());
    let deadline = Instant::now() + Duration::from_secs(30);
    while !release.exists() {
        assert!(Instant::now() < deadline, "chat fixture was not released");
        std::thread::sleep(Duration::from_millis(10));
    }

    println!(
        r#"{{"type":"result","is_error":false,"result":"{{\"answer\":\"fresh answer\"}}"}}"#
    );
}
