use std::io::Read;

// #149: a coordinator that answers a review with a typed amendment; the mode picks the path.
fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("agent:one.txt") && prompt.contains("add_paths"));
    let path = match std::env::args().nth(1).as_deref() {
        Some("dir") => "src",
        Some("secret") => ".env",
        Some("overlap") => "two.txt",
        _ => "extra.txt",
    };
    println!(
        "{{\"type\":\"result\",\"payload\":{{\"verdict\":\"revise\",\"summary\":\"The task writes a file next to its own\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[{{\"task\":\"one.txt\",\"feedback\":\"Add the file the task writes\",\"add_paths\":[\"{path}\"]}}]}}}}"
    );
}
