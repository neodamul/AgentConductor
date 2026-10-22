use std::{process::Command, thread, time::Duration};

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("tree") => {
            let child = Command::new(std::env::current_exe().unwrap())
                .arg("sleep")
                .spawn()
                .unwrap();
            std::fs::write("grandchild.pid", child.id().to_string()).unwrap();
            thread::sleep(Duration::from_secs(120));
        }
        Some("sleep") => thread::sleep(Duration::from_secs(120)),
        _ => std::process::exit(2),
    }
}
