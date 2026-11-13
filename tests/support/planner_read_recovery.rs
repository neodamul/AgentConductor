use std::io::{self, Read};
fn main() {
    let mut prompt = String::new();
    io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("Read-only planning execution context"));
    assert!(prompt.contains("snapshot-relative paths"));
    assert!(std::path::Path::new("base.txt").is_file());
    assert!(!std::path::Path::new(".git").exists());
    let events = [
        r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"g-deny","name":"Glob","input":{"path":"/original/project"}}]}}"#,
        r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"g-deny","is_error":true,"content":"permission denied"}]}}"#,
        r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"r-deny","name":"Read","input":{"file_path":"/original/project/base.txt"}}]}}"#,
        r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"r-deny","is_error":true,"content":"permission denied"}]}}"#,
    ];
    for event in events {
        println!("{event}");
    }
    if std::env::var("AC_PLANNING_RECOVERY").as_deref() != Ok("unresolved") {
        for event in [
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"g-ok","name":"Glob","input":{"pattern":"*"}}]}}"#,
            r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"g-ok","content":"base.txt"}]}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"r-ok","name":"Read","input":{"file_path":"base.txt"}}]}}"#,
            r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"r-ok","content":"base"}]}}"#,
        ] {
            println!("{event}");
        }
    }
    println!(
        "{}",
        r#"{"type":"result","is_error":false,"permission_denials":[{"tool_name":"Glob","tool_use_id":"g-deny"},{"tool_name":"Read","tool_use_id":"r-deny"}],"structured_output":{"tasks":[{"id":"one","agent":"mock","prompt":"Implement one","paths":["one.txt"],"test_command":["cargo","test"]},{"id":"two","agent":"mock","prompt":"Implement two","paths":["two.txt"],"test_command":["cargo","test"]},{"id":"three","agent":"mock","prompt":"Implement three","paths":["three.txt"],"test_command":["cargo","test"]}]}}"#
    );
}
