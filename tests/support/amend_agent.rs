use std::io::Read;

// #149: a coordinator that answers a review with a typed amendment; the mode picks the path.
fn main() {
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("agent:one.txt") && prompt.contains("add_paths"));
    // "drift": the first review adds eight paths, later ones two more (a marker counts rounds).
    if std::env::args().nth(1).as_deref() == Some("drift") {
        let marker = std::path::PathBuf::from(std::env::var_os("AC_DRIFT_MARKER").unwrap());
        let round = marker.exists();
        std::fs::write(&marker, "x").unwrap();
        let paths: Vec<String> = if round {
            (1..=2).map(|i| format!("\"e{i}.txt\"")).collect()
        } else {
            (1..=8).map(|i| format!("\"d{i}.txt\"")).collect()
        };
        println!(
            "{{\"type\":\"result\",\"payload\":{{\"verdict\":\"revise\",\"summary\":\"Widen the scope\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[{{\"task\":\"one.txt\",\"feedback\":\"Add files\",\"add_paths\":[{}]}}]}}}}",
            paths.join(",")
        );
        return;
    }
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
