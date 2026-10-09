use std::io::{Read, Write};

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("echo") => {
            std::io::stderr().write_all(&vec![b'e'; 2 * 1024 * 1024]).unwrap();
            let mut prompt = Vec::new();
            std::io::stdin().read_to_end(&mut prompt).unwrap();
            std::io::stdout().write_all(&prompt).unwrap();
        }
        Some("stubborn") => {
            unsafe extern "C" { fn signal(sig: i32, handler: usize) -> usize; }
            unsafe { signal(2, 1); signal(15, 1); }
            std::fs::write("ready", "yes").unwrap();
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
        Some("fail") => std::process::exit(7),
        Some("tree") => {
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .arg("sleep")
                .spawn()
                .unwrap();
            std::fs::write("grandchild.pid", child.id().to_string()).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
        _ => std::thread::sleep(std::time::Duration::from_secs(60)),
    }
}
