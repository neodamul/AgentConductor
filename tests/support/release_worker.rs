//! Deterministic scoped CLI fixture for packaged-binary acceptance; no model or network.
use std::io::{Read, Write};

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("task") => {
            let mut prompt = String::new();
            std::io::stdin().read_to_string(&mut prompt).unwrap();
            let path = prompt.lines().next().unwrap();
            assert!(["one.txt", "two.txt", "three.txt"].contains(&path));
            println!("{{\"type\":\"session\",\"session_id\":\"release-{path}\"}}");
            std::io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(2));
            std::fs::write(path, "implemented\n").unwrap();
            println!("{{\"type\":\"result\",\"payload\":{{\"summary\":\"implemented\"}}}}");
        }
        Some("test") => {
            for path in ["one.txt", "two.txt", "three.txt"] {
                assert_eq!(std::fs::read_to_string(path).unwrap(), "implemented\n");
            }
            println!("all scoped files verified");
        }
        _ => std::process::exit(2),
    }
}
