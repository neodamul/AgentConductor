use std::io::Read;

fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("agent:one.txt") && prompt.contains("add_paths"));
    let path = match std::env::args().nth(1).as_deref() {
        Some("directory") => "src",
        Some("secret") => ".env",
        Some("overlap") => "two.txt",
        _ => "extra.txt",
    };
    println!("{{\"type\":\"result\",\"payload\":{{\"verdict\":\"revise\",\"summary\":\"The task writes an additional file\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[{{\"task\":\"one.txt\",\"feedback\":\"Include the additional file\",\"add_paths\":[\"{path}\"]}}]}}}}");
}
