use std::{io::Read, thread, time::Duration};

fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("Answer the last operator message."));
    if let Ok(milliseconds) = std::env::var("AC_CHAT_FIXTURE_DELAY_MS") {
        thread::sleep(Duration::from_millis(milliseconds.parse().unwrap()));
    }
    println!(
        r#"{{"type":"result","payload":{{"answer":"PACKAGE_CHAT_ANSWER"}}}}"#
    );
}
