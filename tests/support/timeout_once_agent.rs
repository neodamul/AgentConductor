use std::io::{Read, Write};

fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    let path = prompt.lines().next().unwrap().trim();
    println!("{{\"type\":\"session\",\"session_id\":\"timeout-{path}\"}}");
    std::io::stdout().flush().unwrap();
    if path == "one.txt" {
        let marker = std::path::PathBuf::from(std::env::var_os("MOCK_MARKERS").unwrap())
            .join("timed-out-once");
        if !marker.exists() {
            std::fs::write(marker, "timed out").unwrap();
            std::thread::sleep(std::time::Duration::from_secs(8));
        }
    }
    std::fs::write(path, "implemented\n").unwrap();
    println!("{{\"type\":\"result\"}}");
}
