use std::io::Write;

pub fn run(mode: &str) {
    match mode {
        "control-final" => {
            use std::io::BufRead;
            let mut line = String::new();
            std::io::stdin().lock().read_line(&mut line).unwrap();
            println!("{{\"type\":\"result\",\"is_error\":false}}");
        }
        "guard-control" => {
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
        "control" | "control-malformed" => {
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
            if mode == "control-malformed" {
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
        _ => unreachable!(),
    }
}
