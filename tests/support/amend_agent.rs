use std::io::Read;

fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("agent:one.txt") && prompt.contains("add_paths"));
    match std::env::args().nth(1).as_deref() {
        Some("test-command") => {
            println!(r#"{{"type":"result","payload":{{"verdict":"revise","summary":"Use a test command that works in this project.","evidence":["agent:one.txt"],"revisions":[{{"task":"one.txt","feedback":"The original test command fails in this project; retain an equally strong check.","test_command":["true"]}}]}}}}"#);
            return;
        }
        Some("request-answer") => {
            assert!(prompt.contains("Operator request") && prompt.contains("reconcile"));
            println!("{{\"type\":\"result\",\"payload\":{{\"verdict\":\"accept\",\"summary\":\"Reconciled the run\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[]}}}}");
            return;
        }
        Some("request-fail") => std::process::exit(3),
        Some("drift") => {
            println!(r#"{{"type":"result","payload":{{"verdict":"revise","summary":"Split a wider scope across two changes","evidence":["agent:one.txt","agent:two.txt"],"revisions":[{{"task":"one.txt","feedback":"Add the first files","add_paths":["extra-0.txt","extra-1.txt","extra-2.txt","extra-3.txt","extra-4.txt"]}},{{"task":"two.txt","feedback":"Add more files","add_paths":["extra-5.txt","extra-6.txt","extra-7.txt","extra-8.txt"]}}]}}}}"#);
            return;
        }
        _ => {}
    }
    let path = match std::env::args().nth(1).as_deref() {
        Some("directory" | "dir") => "src",
        Some("secret") => ".env",
        Some("overlap") => "two.txt",
        _ => "extra.txt",
    };
    println!("{{\"type\":\"result\",\"payload\":{{\"verdict\":\"revise\",\"summary\":\"The task writes an additional file\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[{{\"task\":\"one.txt\",\"feedback\":\"Include the additional file\",\"add_paths\":[\"{path}\"]}}]}}}}");
}
