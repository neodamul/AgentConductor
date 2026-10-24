use std::io::{Read, Write};

pub fn run(mode: &str) {
    match mode {
        "plan-phases" => {
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
        "plan" | "plan-hostile" | "plan-roles" => {
            let mut prompt = String::new();
            std::io::stdin().read_to_string(&mut prompt).unwrap();
            if mode == "plan-hostile" {
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
                let role = if mode == "plan-roles" {
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
        "review" | "review-unknown" | "review-slow" | "review-write" | "review-accept"
        | "review-command" => {
            if let Ok(model) = std::env::var("AC_EXPECTED_MODEL") {
                let args: Vec<_> = std::env::args().collect();
                assert!(
                    args.windows(2).any(|pair| pair == ["--model", model.as_str()]),
                    "fallback model missing from argv: {:?}",
                    args
                );
            }
            let mut prompt = String::new();
            std::io::stdin().read_to_string(&mut prompt).unwrap();
            assert!(prompt.contains("agent:one.txt"));
            assert!(prompt.contains("untrusted data"));
            if mode == "review-accept" {
                println!(
                    "{{\"type\":\"result\",\"payload\":{{\"verdict\":\"accept\",\"summary\":\"Evidence verified\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[]}}}}"
                );
                return;
            }
            // A test command that cannot work is replaced by one that verifies the same file.
            if mode == "review-command" {
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
            if mode == "review-slow" {
                println!(
                    "{{\"type\":\"session\",\"session_id\":\"review-{}\"}}",
                    std::process::id()
                );
                std::io::stdout().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
            if mode == "review-write" {
                std::fs::write("one.txt", "invalid coordinator edit\n").unwrap();
            }
            let task = if mode == "review-unknown" {
                "invented"
            } else {
                "one.txt"
            };
            println!(
                "{{\"type\":\"result\",\"payload\":{{\"verdict\":\"revise\",\"summary\":\"Repair failed implementation\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[{{\"task\":\"{task}\",\"feedback\":\"Repair failed task\"}}]}}}}"
            );
        }
        _ => unreachable!(),
    }
}
