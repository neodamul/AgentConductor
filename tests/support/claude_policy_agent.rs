use std::io::{self, BufRead, Read, Write};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "test") {
        return;
    }
    if args.iter().any(|a| a == "--version") {
        println!("policy-fixture 1");
        return;
    }
    let mode = std::env::var("AC_POLICY_MODE").unwrap_or("bypassPermissions".into());
    let mut prompt = String::new();
    if args.iter().any(|a| a == "--input-format") {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(line.contains("control_request") && line.contains("initialize"));
        assert!(!line.contains("\"type\":\"user\""));
        if let Ok(path) = std::env::var("AC_POLICY_CAPTURE") {
            std::fs::write(path, args.join("\n")).unwrap();
        }
        let id = if line.contains("permission-init") {
            "permission-init"
        } else {
            "catalog-init"
        };
        let effective = if [
            "denied",
            "partial-denied",
            "delayed-denied",
            "quota-noise-denied",
            "missing",
        ]
        .contains(&mode.as_str())
        {
            "bypassPermissions"
        } else {
            &mode
        };
        println!(
            "{{\"type\":\"control_response\",\"response\":{{\"request_id\":\"{id}\",\"subtype\":\"success\",\"response\":{{\"models\":[{{\"value\":\"fixture-model\"}}],\"current_permission_mode\":\"{effective}\"}}}}}}"
        );
        io::stdout().flush().unwrap();
        if id == "catalog-init" {
            return;
        }
        if reader.read_line(&mut prompt).unwrap() == 0 {
            return;
        }
        assert!(prompt.contains("\"type\":\"user\""));
    } else {
        io::stdin().read_to_string(&mut prompt).unwrap();
    }
    if let Ok(marker) = std::env::var("AC_POLICY_USER_MARKER") {
        std::fs::write(marker, "generation requested").unwrap();
    }
    let path = "one.txt";
    if mode != "missing" {
        println!(
            "{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"fixture-owned\",\"permissionMode\":\"{}\"}}",
            if mode == "denied"
                || mode == "partial-denied"
                || mode == "delayed-denied"
                || mode == "quota-noise-denied"
            {
                "bypassPermissions"
            } else {
                &mode
            }
        );
        io::stdout().flush().unwrap();
    }
    if mode == "default" {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        std::fs::write("late-write.txt", "must be cancelled").unwrap();
    }
    if mode == "partial-denied" {
        std::fs::write(path, "partial\n").unwrap();
    }
    if mode == "delayed-denied" {
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    if mode == "quota-noise-denied" {
        eprintln!("Usage limit reached");
    }
    if [
        "denied",
        "partial-denied",
        "delayed-denied",
        "quota-noise-denied",
    ]
    .contains(&mode.as_str())
    {
        println!(
            "{{\"type\":\"result\",\"is_error\":false,\"permission_denials\":[{{\"tool_name\":\"Write\",\"tool_input\":{{\"content\":\"private marker\"}}}}]}}"
        );
    } else {
        if mode == "bypassPermissions" {
            std::fs::write(path, "done\n").unwrap();
        }
        println!("{{\"type\":\"result\",\"is_error\":false,\"permission_denials\":[]}}");
    }
}
