use std::io::{Read, Write};

#[path = "mock_agent/control.rs"]
mod control;
#[path = "mock_agent/planning.rs"]
mod planning;
#[path = "mock_agent/session.rs"]
mod session;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some(mode @ ("control-final" | "guard-control" | "control" | "control-malformed")) => {
            control::run(mode)
        }
        Some("--version") => println!("mock 0.1"),
        Some("models") => println!("local/one\nlocal/two"),
        Some("-p") => session::run(),
        Some("app-server") => {
            use std::io::BufRead;
            for line in std::io::stdin().lock().lines() {
                let line = line.unwrap();
                assert!(!line.contains("turn/start"));
                if line.contains("model/list") {
                    println!(
                        "{{\"id\":2,\"result\":{{\"data\":[{{\"model\":\"local/one\"}},{{\"model\":\"local/two\"}}],\"nextCursor\":null}}}}"
                    );
                    break;
                }
                if line.contains("clientInfo") {
                    println!("{{\"id\":1,\"result\":{{}}}}");
                }
                std::io::stdout().flush().unwrap();
            }
        }
        Some("echo") => {
            std::io::stderr()
                .write_all(&vec![b'e'; 2 * 1024 * 1024])
                .unwrap();
            let mut prompt = Vec::new();
            std::io::stdin().read_to_end(&mut prompt).unwrap();
            std::io::stdout().write_all(&prompt).unwrap();
        }
        Some("stubborn") => {
            unsafe extern "C" {
                fn signal(sig: i32, handler: usize) -> usize;
            }
            unsafe {
                signal(2, 1);
                signal(15, 1);
            }
            std::fs::write("ready", "yes").unwrap();
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
        Some("fail") => std::process::exit(7),
        // DD-132 fixture: integration fails once, then passes after the coordinator's repair.
        Some("test-repaired") => {
            let marker = std::path::Path::new("../test-repaired-marker");
            if !marker.exists() {
                std::fs::write(marker, "failed once").unwrap();
                std::process::exit(3);
            }
            println!("integration tests passed");
        }
        Some("test") | Some("test-wait") => {
            if std::env::args().nth(1).as_deref() == Some("test-wait") {
                println!("integration test started");
                std::io::stdout().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
            for path in ["one.txt", "two.txt", "three.txt"] {
                assert!(std::path::Path::new(path).exists());
            }
            println!("integration tests passed");
        }
        Some("dependency-test") => {
            let one = std::fs::read_to_string("one.txt").unwrap();
            let two = std::fs::read_to_string("two.txt").unwrap();
            let three = std::fs::read_to_string("three.txt").unwrap();
            assert_eq!(two, format!("observed {one}"));
            assert_eq!(three, format!("documented {two}"));
            println!("dependency consistency passed");
        }
        Some(
            mode @ ("plan-phases" | "plan" | "plan-hostile" | "plan-roles" | "review"
            | "review-unknown" | "review-slow" | "review-write" | "review-accept"
            | "review-command"),
        ) => planning::run(mode),
        Some("quota-long")
        | Some("quota-partial-long")
        | Some("quota-slow")
        | Some("quota-partial")
        | Some("task-preserve")
        | Some("task")
        | Some("task-stray")
        | Some("task-slow")
        | Some("dependency")
        | Some("quota")
        | Some("quota-once")
        | Some("quota-known") => {
            let mut prompt = String::new();
            std::io::stdin().read_to_string(&mut prompt).unwrap();
            let original = prompt.lines().next().unwrap().trim();
            let path = if original == "fail" && prompt.contains("Repair failed task") {
                prompt
                    .lines()
                    .find_map(|line| line.strip_prefix("Owned paths: "))
                    .unwrap()
                    .trim()
            } else {
                original
            };
            use std::hash::{Hash, Hasher};
            let mut identity = std::collections::hash_map::DefaultHasher::new();
            std::env::current_dir().unwrap().hash(&mut identity);
            println!(
                "{{\"type\":\"session\",\"session_id\":\"mock-{:x}-{path}\"}}",
                identity.finish()
            );
            std::io::stdout().flush().unwrap();
            let mode = std::env::args().nth(1).unwrap();
            let marker = std::env::var("MOCK_MARKERS")
                .map(|dir| std::path::PathBuf::from(dir).join(format!("marker-{path}")))
                .ok();
            if mode == "quota-slow" {
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
            if mode == "quota-partial" || mode == "quota-partial-long" {
                std::fs::write(path, "partial\n").unwrap();
            }
            if mode == "quota-long"
                || mode == "quota-partial-long"
                || mode == "quota-slow"
                || mode == "quota-partial"
                || mode == "quota"
                || mode == "quota-known"
                || (mode == "quota-once" && !marker.as_ref().unwrap().exists())
            {
                if let Some(marker) = marker {
                    std::fs::write(marker, "restricted").unwrap();
                }
                let reset = if mode == "quota-known" {
                    ",\"resetsAt\":200"
                } else if mode == "quota-long" || mode == "quota-partial-long" {
                    ",\"resetsAt\":4102444800"
                } else {
                    ""
                };
                println!(
                    "{{\"type\":\"error\",\"error\":{{\"code\":\"usage_limit_reached\",\"message\":\"Usage limit reached\"{reset}}}}}"
                );
                std::process::exit(1);
            }
            std::thread::sleep(std::time::Duration::from_millis(if mode == "task-slow" {
                3000
            } else {
                500
            }));
            if path == "fail" {
                std::process::exit(7);
            }
            let content = if mode == "task-preserve" {
                format!(
                    "{}implemented\n",
                    std::fs::read_to_string(path).expect("partial work lost")
                )
            } else if mode == "dependency" {
                if path != "one.txt" && prompt.contains("\"attempt\": 2") {
                    assert!(
                        std::path::Path::new(path).exists(),
                        "previous owned partial file was discarded"
                    );
                }
                match path {
                    "one.txt" => {
                        if prompt.contains("Use version two") {
                            "v2\n".into()
                        } else {
                            "v1\n".into()
                        }
                    }
                    "two.txt" => {
                        format!("observed {}", std::fs::read_to_string("one.txt").unwrap())
                    }
                    "three.txt" => {
                        format!("documented {}", std::fs::read_to_string("two.txt").unwrap())
                    }
                    _ => panic!("unknown owned path"),
                }
            } else {
                "implemented\n".into()
            };
            std::fs::write(path, content).unwrap();
            // A CLI that saves its own plan inside the project, outside every owned path.
            if mode == "task-stray" {
                std::fs::create_dir_all(".aipro/plans").unwrap();
                std::fs::write(format!(".aipro/plans/{path}.md"), "plan\n").unwrap();
            }
            println!("{{\"type\":\"result\",\"payload\":{{\"summary\":\"implemented\"}}}}");
        }
        Some("live-events") => {
            use std::io::Write;
            print!(
                "{{\"type\":\"session\",\"session_id\":\"live-session\"}}\n{{\"type\":\"progress\",\"text\":\"🦀\"}}\n"
            );
            std::io::stdout().flush().unwrap();
            while !std::path::Path::new("release").exists() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            print!("{{\"type\":\"result\"}}");
        }
        Some("live-invalid") => {
            use std::io::Write;
            println!("invalid-json");
            std::io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
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
