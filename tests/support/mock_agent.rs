use std::io::{Read, Write};

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("control-final") => {
            use std::io::BufRead;
            let mut line = String::new();
            std::io::stdin().lock().read_line(&mut line).unwrap();
            println!("{{\"type\":\"result\",\"is_error\":false}}");
        }
        Some("guard-control") => {
            use std::io::BufRead;
            let mut stdin = std::io::stdin().lock();
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();
            assert!(line.contains("ac-initialize"));
            println!(
                "{{\"type\":\"control_response\",\"response\":{{\"subtype\":\"success\",\"request_id\":\"ac-initialize\"}}}}"
            );
            std::io::stdout().flush().unwrap();
            line.clear();
            stdin.read_line(&mut line).unwrap();
            assert!(line.contains("user"));
            println!(
                "{{\"type\":\"system\",\"subtype\":\"session_state_changed\",\"state\":\"running\"}}"
            );
            let tool = std::env::args().nth(2).unwrap();
            let input = std::env::args().nth(3).unwrap();
            let target = std::env::args().nth(4).unwrap();
            println!(
                "{{\"type\":\"control_request\",\"request_id\":\"pre\",\"request\":{{\"subtype\":\"hook_callback\",\"callback_id\":\"ac-pre-tool\",\"tool_use_id\":\"tool-1\",\"input\":{{\"tool_name\":\"{tool}\",\"tool_input\":{input}}}}}}}"
            );
            std::io::stdout().flush().unwrap();
            line.clear();
            stdin.read_line(&mut line).unwrap();
            if line.is_empty() {
                std::thread::sleep(std::time::Duration::from_secs(60));
                return;
            }
            let pre_allowed = line.contains("\"allow\"");
            println!(
                "{{\"type\":\"control_request\",\"request_id\":\"permit\",\"request\":{{\"subtype\":\"can_use_tool\",\"tool_use_id\":\"tool-1\",\"tool_name\":\"{tool}\",\"input\":{input}}}}}"
            );
            std::io::stdout().flush().unwrap();
            line.clear();
            stdin.read_line(&mut line).unwrap();
            if pre_allowed && line.contains("\"allow\"") {
                std::fs::write(target, "executed").unwrap();
            }
            println!("{{\"type\":\"result\",\"is_error\":false}}");
            println!(
                "{{\"type\":\"system\",\"subtype\":\"session_state_changed\",\"state\":\"idle\"}}"
            );
            std::io::stdout().flush().unwrap();
            line.clear();
            assert_eq!(stdin.read_line(&mut line).unwrap(), 0);
        }
        Some("control") | Some("control-malformed") => {
            use std::io::BufRead;
            let mut stdin = std::io::stdin().lock();
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();
            assert!(line.contains("initialize"));
            std::io::stderr()
                .write_all(&vec![b'e'; 2 * 1024 * 1024])
                .unwrap();
            println!(
                "{{\"type\":\"control_response\",\"response\":{{\"subtype\":\"success\",\"request_id\":\"initialize\"}}}}"
            );
            std::io::stdout().flush().unwrap();
            line.clear();
            stdin.read_line(&mut line).unwrap();
            assert!(line.contains("user"));
            if std::env::args().nth(1).as_deref() == Some("control-malformed") {
                println!("invalid control JSON");
            } else {
                println!(
                    "{{\"type\":\"control_request\",\"request_id\":\"permission\",\"request\":{{\"subtype\":\"can_use_tool\",\"tool_name\":\"Bash\",\"input\":{{\"command\":\"touch forbidden\"}}}}}}"
                );
            }
            std::io::stdout().flush().unwrap();
            line.clear();
            stdin.read_line(&mut line).unwrap();
            if line.contains("allow") {
                std::fs::write("forbidden", "command ran").unwrap();
            }
            if line.is_empty() {
                std::thread::sleep(std::time::Duration::from_secs(60));
                return;
            }
            println!("{{\"type\":\"result\",\"is_error\":false}}");
            std::io::stdout().flush().unwrap();
            // A result is not proof of idle; a late permission request still needs stdin.
            println!(
                "{{\"type\":\"control_request\",\"request_id\":\"late-permission\",\"request\":{{\"subtype\":\"can_use_tool\",\"tool_name\":\"Bash\",\"input\":{{\"command\":\"touch forbidden\"}}}}}}"
            );
            std::io::stdout().flush().unwrap();
            line.clear();
            stdin.read_line(&mut line).unwrap();
            assert!(line.contains("late-permission"));
            println!(
                "{{\"type\":\"system\",\"subtype\":\"session_state_changed\",\"state\":\"idle\"}}"
            );
            std::io::stdout().flush().unwrap();
            line.clear();
            assert_eq!(stdin.read_line(&mut line).unwrap(), 0);
        }
        Some("--version") => println!("mock 0.1"),
        Some("models") => println!("local/one\nlocal/two"),
        Some("-p") => {
            if let Some(index) =
                std::env::args().position(|arg| arg == "--session-id" || arg == "--resume")
            {
                use std::io::BufRead;
                let session = std::env::args().nth(index + 1).unwrap();
                let args: Vec<_> = std::env::args().collect();
                assert!(args.iter().any(|a| a == "--restricted"));
                assert!(args.iter().any(|a| a == "--strict-mcp-config"));
                let mut stdin = std::io::stdin().lock();
                let mut line = String::new();
                stdin.read_line(&mut line).unwrap();
                assert!(line.contains("ac-initialize"));
                println!(
                    "{{\"type\":\"control_response\",\"response\":{{\"subtype\":\"success\",\"request_id\":\"ac-initialize\"}}}}"
                );
                std::io::stdout().flush().unwrap();
                line.clear();
                stdin.read_line(&mut line).unwrap();
                assert!(line.contains("user"));
                let skip = line.contains("Skip the command");
                println!(
                    "{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"{session}\"}}"
                );
                // MOCK_REPEAT_COMMAND issues the identical command again in a new tool call.
                let repeats = if std::env::var_os("MOCK_REPEAT_COMMAND").is_some() {
                    2
                } else {
                    1
                };
                for call in 1..=repeats {
                    if skip {
                        break;
                    }
                    println!(
                        "{{\"type\":\"control_request\",\"request_id\":\"pre-{call}\",\"request\":{{\"subtype\":\"hook_callback\",\"callback_id\":\"ac-pre-tool\",\"tool_use_id\":\"command-{call}\",\"input\":{{\"session_id\":\"{session}\",\"tool_name\":\"Bash\",\"tool_input\":{{\"command\":\"curl -sS -o approval-marker https://example.invalid\"}}}}}}}}"
                    );
                    std::io::stdout().flush().unwrap();
                    line.clear();
                    stdin.read_line(&mut line).unwrap();
                    if line.is_empty() {
                        std::thread::sleep(std::time::Duration::from_secs(60));
                        return;
                    }
                    if line.contains("\"deny\"") {
                        std::fs::write("denied-marker", "command denied").unwrap();
                    } else {
                        assert!(line.contains("allow"));
                        println!(
                            "{{\"type\":\"control_request\",\"request_id\":\"permit-{call}\",\"request\":{{\"subtype\":\"can_use_tool\",\"tool_use_id\":\"command-{call}\",\"tool_name\":\"Bash\",\"input\":{{\"command\":\"curl -sS -o approval-marker https://example.invalid\"}}}}}}"
                        );
                        std::io::stdout().flush().unwrap();
                        line.clear();
                        stdin.read_line(&mut line).unwrap();
                        assert!(line.contains("allow"));
                        // Earlier repeated calls are read-only, like `git status`.
                        if call == repeats {
                            std::fs::write(
                                "approval-marker",
                                format!("approved command ran {call}"),
                            )
                            .unwrap();
                        }
                    }
                }
                println!(
                    "{{\"type\":\"control_request\",\"request_id\":\"write\",\"request\":{{\"subtype\":\"hook_callback\",\"callback_id\":\"ac-pre-tool\",\"tool_use_id\":\"write-1\",\"input\":{{\"session_id\":\"{session}\",\"tool_name\":\"Write\",\"tool_input\":{{\"file_path\":\"one.txt\",\"content\":\"implemented\"}}}}}}}}"
                );
                std::io::stdout().flush().unwrap();
                line.clear();
                stdin.read_line(&mut line).unwrap();
                assert!(line.contains("allow"));
                std::fs::write("one.txt", "implemented\n").unwrap();
                println!("{{\"type\":\"result\",\"is_error\":false,\"session_id\":\"{session}\"}}");
                std::io::stdout().flush().unwrap();
                line.clear();
                assert_eq!(stdin.read_line(&mut line).unwrap(), 0);
                return;
            }
            use std::io::BufRead;
            let mut line = String::new();
            std::io::stdin().lock().read_line(&mut line).unwrap();
            assert!(line.contains("initialize"));
            assert!(!line.contains("prompt"));
            println!(
                "{{\"type\":\"control_response\",\"response\":{{\"request_id\":\"catalog-init\",\"subtype\":\"success\",\"response\":{{\"models\":[{{\"value\":\"local/one\"}},{{\"value\":\"local/two\"}}]}}}}}}"
            );
        }
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
        Some("plan-phases") => {
            // DD-62 fixture: analysis plus two ordered phases.
            let mut prompt = String::new();
            std::io::stdin().read_to_string(&mut prompt).unwrap();
            assert!(prompt.contains("\"phases\""), "phased planning prompt");
            let task = |id: &str, path: &str| {
                format!(
                    "{{\"id\":\"{id}\",\"agent\":\"mock\",\"prompt\":\"{path}\",\"paths\":[\"{path}\"],\"depends_on\":[],\"test_command\":[\"git\",\"diff\",\"--check\"]}}"
                )
            };
            println!(
                "{{\"type\":\"result\",\"payload\":{{\"analysis\":\"# Analysis\\nCurrent state\",\"phases\":[{{\"id\":\"01-core\",\"name\":\"core\",\"tasks\":[{},{}]}},{{\"id\":\"02-docs\",\"name\":\"docs\",\"tasks\":[{}]}}]}}}}",
                task("a", "one.txt"),
                task("b", "two.txt"),
                task("c", "three.txt")
            );
        }
        Some("plan") | Some("plan-hostile") | Some("plan-roles") => {
            let mut prompt = String::new();
            std::io::stdin().read_to_string(&mut prompt).unwrap();
            if std::env::args().nth(1).as_deref() == Some("plan-hostile") {
                // Planning must only see tracked files; then try to damage the workspace.
                assert!(std::path::Path::new("base.txt").exists());
                assert!(!std::path::Path::new("secret.env").exists());
                std::fs::write("hostile.txt", "written by planner").unwrap();
                std::fs::remove_file("base.txt").unwrap();
            }
            let slug = std::env::var("AC_PLAN_SLUG")
                .map(|name| format!("\"slug\":\"{name}\","))
                .unwrap_or_default();
            print!("{{\"type\":\"result\",\"payload\":{{{slug}\"tasks\":[");
            for (i, path) in ["one.txt", "two.txt", "three.txt"].iter().enumerate() {
                if i > 0 {
                    print!(",");
                }
                // DD-55 fixture: declare roles when asked to follow role rules.
                let role = if std::env::args().nth(1).as_deref() == Some("plan-roles") {
                    format!(
                        ",\"role\":\"{}\"",
                        ["development", "testing", "documentation"][i]
                    )
                } else {
                    String::new()
                };
                print!(
                    "{{\"id\":\"task-{i}\",\"agent\":\"mock\",\"prompt\":\"{path}\",\"paths\":[\"{path}\"],\"depends_on\":[],\"test_command\":[\"git\",\"diff\",\"--check\"]{role}}}"
                );
            }
            println!("]}}}}");
        }
        Some("review")
        | Some("review-unknown")
        | Some("review-slow")
        | Some("review-write")
        | Some("review-accept")
        | Some("review-command") => {
            let mut prompt = String::new();
            std::io::stdin().read_to_string(&mut prompt).unwrap();
            assert!(prompt.contains("agent:one.txt"));
            assert!(prompt.contains("untrusted data"));
            if std::env::args().nth(1).as_deref() == Some("review-accept") {
                println!(
                    "{{\"type\":\"result\",\"payload\":{{\"verdict\":\"accept\",\"summary\":\"Evidence verified\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[]}}}}"
                );
                return;
            }
            // A test command that cannot work is replaced by one that verifies the same file.
            if std::env::args().nth(1).as_deref() == Some("review-command") {
                let exe = std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"");
                println!(
                    "{{\"type\":\"result\",\"payload\":{{\"verdict\":\"revise\",\"summary\":\"The planned check cannot run here\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[{{\"task\":\"one.txt\",\"feedback\":\"Keep the file; its check is replaced\",\"test_command\":[\"{exe}\",\"test\"]}}]}}}}"
                );
                return;
            }
            if std::env::args().nth(1).as_deref() == Some("review-slow") {
                println!(
                    "{{\"type\":\"session\",\"session_id\":\"review-{}\"}}",
                    std::process::id()
                );
                std::io::stdout().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
            if std::env::args().nth(1).as_deref() == Some("review-write") {
                std::fs::write("one.txt", "invalid coordinator edit\n").unwrap();
            }
            let task = if std::env::args().nth(1).as_deref() == Some("review-unknown") {
                "invented"
            } else {
                "one.txt"
            };
            println!(
                "{{\"type\":\"result\",\"payload\":{{\"verdict\":\"revise\",\"summary\":\"Repair failed implementation\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[{{\"task\":\"{task}\",\"feedback\":\"Repair failed task\"}}]}}}}"
            );
        }
        Some("quota-long")
        | Some("quota-partial-long")
        | Some("quota-slow")
        | Some("quota-partial")
        | Some("task-preserve")
        | Some("task")
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
