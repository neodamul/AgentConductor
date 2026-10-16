use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub fn run(args: &[String]) {
    let session = args.first().expect("session id");
    let run = std::env::var("AC_CHECKPOINT_RUN_DIR").unwrap();
    let address_file = std::env::var("AC_CHECKPOINT_ADDRESS_FILE").unwrap();
    let address: Value = serde_json::from_slice(&fs::read(&address_file).unwrap()).unwrap();
    assert_eq!(address["session_id"].as_str(), Some(session.as_str()));
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("Cooperative checkpoint contract"));
    let require_continuation = std::env::var_os("MOCK_REQUIRE_CONTINUATION").is_some();
    let continuation = address["attempt"].as_u64().unwrap_or(1) > 1 && require_continuation;
    if continuation {
        assert!(
            prompt.contains("Use revised content") && prompt.contains("local checkpoint ledger")
        );
        assert_eq!(
            fs::read_to_string(address["task"].as_str().unwrap()).unwrap(),
            "partial\n"
        );
    }
    let executable = std::env::var("AC_CHECKPOINT_EXECUTABLE").unwrap();
    let artifacts = Path::new(&address_file).parent().unwrap();
    let command = |name: &str, packet: &Value| -> Value {
        let path = artifacts.join(format!("{name}.json"));
        fs::write(&path, serde_json::to_vec(packet).unwrap()).unwrap();
        let output = Command::new(&executable)
            .args([name, &run, path.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    };
    let mut sequence = 1;
    let report = |sequence, phase: &str| {
        command(
            "checkpoint-report",
            &json!({"address":address,"sequence":sequence,"reported_at_ms":now_ms(),"phase":phase,"summary":"Checkpoint fixture reports its real activity","next_step":"Inspect addressed directives"}),
        );
    };
    assert!(
        command("checkpoint-poll", &address)
            .as_array()
            .unwrap()
            .is_empty()
    );
    report(sequence, "prepared");
    fs::write(artifacts.join("checkpoint-ready"), "ready").unwrap();
    let deadline = Instant::now() + Duration::from_secs(12);
    wait_for(Path::new(&std::env::var("MOCK_RELEASE").unwrap()), deadline);
    let directives = command("checkpoint-poll", &address);
    let mut content = if continuation {
        "revised\n"
    } else {
        "implemented\n"
    };
    for directive in directives.as_array().unwrap() {
        assert_eq!(directive["address"], address);
        let action = directive["action"].as_str().unwrap();
        assert!(matches!(action, "rework" | "pause" | "handoff"));
        assert_eq!(
            directive["instruction"],
            if action == "rework" {
                "Use revised content"
            } else {
                "Relinquish at checkpoint"
            }
        );
        let ack_mode = std::env::var("MOCK_ACK_MODE").unwrap_or_else(|_| "completed".into());
        let mut ack = json!({"id":directive["id"],"address":address,"state":"accepted","reported_at_ms":now_ms(),"reason":"Safe checkpoint reached"});
        if ack_mode != "pending" {
            command("checkpoint-ack", &ack);
        }
        content = if action == "rework" {
            "revised\n"
        } else {
            "partial\n"
        };
        ack["state"] = json!("completed");
        ack["reported_at_ms"] = json!(now_ms());
        if ack_mode == "completed" {
            command("checkpoint-ack", &ack);
        }
        if action != "rework" || require_continuation {
            if action == "rework" {
                content = "partial\n";
                fs::write(directive["address"]["task"].as_str().unwrap(), content).unwrap();
            }
            fs::write(
                artifacts.join("checkpoint-relinquished"),
                "acknowledged, process still live",
            )
            .unwrap();
            wait_for(
                Path::new(&std::env::var("MOCK_EXIT_RELEASE").unwrap()),
                deadline,
            );
        }
    }
    fs::write(address["task"].as_str().unwrap(), content).unwrap();
    sequence += 1;
    report(sequence, "completed");
    let returned_session = if std::env::var_os("MOCK_BAD_SESSION").is_some() {
        "foreign-session"
    } else {
        session
    };
    println!(
        "{}",
        json!({"type":"session","session_id":returned_session})
    );
    println!(
        "{}",
        json!({"type":"result","payload":{"summary":"Checkpoint fixture completed"}})
    );
    std::io::stdout().flush().unwrap();
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn wait_for(path: &Path, deadline: Instant) {
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "checkpoint fixture timeout: {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
