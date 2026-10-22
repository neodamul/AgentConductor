use std::io::Read;

fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("agent:one.txt") && prompt.contains("add_paths"));
    match std::env::args().nth(1).as_deref() {
        Some("request-answer") => {
            assert!(prompt.contains("Operator request") && prompt.contains("reconcile"));
            println!("{{\"type\":\"result\",\"payload\":{{\"verdict\":\"accept\",\"summary\":\"Reconciled the run\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[]}}}}");
            return;
        }
        Some("request-fail") => std::process::exit(3),
        _ => {}
    }
    let path = match std::env::args().nth(1).as_deref() {
        Some("directory") => "src",
        Some("secret") => ".env",
        Some("overlap") => "two.txt",
        _ => "extra.txt",
    };
    println!("{{\"type\":\"result\",\"payload\":{{\"verdict\":\"revise\",\"summary\":\"The task writes an additional file\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[{{\"task\":\"one.txt\",\"feedback\":\"Include the additional file\",\"add_paths\":[\"{path}\"]}}]}}}}");
}
