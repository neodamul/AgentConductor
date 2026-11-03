use std::io::{Read, Write};

fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    let task = prompt
        .split_once("\n# Task\n")
        .map_or(prompt.as_str(), |(_, task)| task)
        .split("\n# Format\n")
        .next()
        .unwrap();
    let path = task.lines().find(|line| !line.trim().is_empty()).unwrap().trim();
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
