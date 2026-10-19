use std::{fs, io::Read};

fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    let marker = std::env::var("AC_PLAN_REPAIR_MARKER").unwrap();
    let count = fs::read_to_string(&marker).unwrap_or_default().parse::<u8>().unwrap_or(0);
    fs::write(&marker, (count + 1).to_string()).unwrap();
    let paths = if count == 0 { ("[\"score:\"]", "[\"score:\"]") } else {
        assert!(prompt.contains("unsafe or overlapping scope"));
        ("[\"selection.rs\"]", "[\"rules.rs\"]")
    };
    println!(r#"{{"type":"result","structured_output":{{"analysis":"analysis","phases":[{{"id":"01-game-selection","name":"selection","tasks":[{{"id":"a","agent":"claude","prompt":"a","paths":{} ,"depends_on":[],"test_command":["true"]}},{{"id":"b","agent":"claude","prompt":"b","paths":{},"depends_on":[],"test_command":["true"]}}]}}]}}}}"#, paths.0, paths.1);
}
