use std::io::Write;

pub fn run() {
    if let Some(index) = std::env::args().position(|arg| arg == "--session-id" || arg == "--resume")
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
        println!("{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"{session}\"}}");
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
                    std::fs::write("approval-marker", format!("approved command ran {call}"))
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
