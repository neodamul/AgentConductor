use std::io::{Read, Write};

// GitHub #625: the first attempt of every task fails (a transient CLI or gateway failure); later attempts succeed.
fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    let path = prompt.split("\n# Task\n\n").nth(1).unwrap_or(&prompt).lines().next().unwrap().trim();
    println!("{{\"type\":\"session\",\"session_id\":\"fail-once-{path}\"}}");
    std::io::stdout().flush().unwrap();
    let marker = std::path::PathBuf::from(std::env::var_os("MOCK_MARKERS").unwrap()).join(format!("marker-{path}"));
    if !marker.exists() {
        std::fs::write(marker, "failed once").unwrap();
        std::process::exit(7);
    }
    std::fs::write(path, "implemented\n").unwrap();
    println!("{{\"type\":\"result\"}}");
}
