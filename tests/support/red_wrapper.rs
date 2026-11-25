fn main() {
    use std::io::Read;
    match std::env::args().nth(1).as_deref() {
        Some("test") => {
            let actual = std::fs::read_to_string("one.txt").unwrap();
            assert_ne!(actual.trim(), "finished feature");
            println!("{}", "test output ".repeat(500));
            println!("red ok: controlled assertion failed because the feature is absent");
        }
        mode => {
            let mut prompt = String::new();
            std::io::stdin().read_to_string(&mut prompt).unwrap();
            assert!(prompt.contains("red ok: controlled assertion failed"));
            let (verdict, evidence) = match mode {
                Some("accept") => ("accept", "test:one.txt"),
                Some("omit") => ("accept", "agent:one.txt"),
                _ => ("blocked", "test:one.txt"),
            };
            println!(
                r#"{{"type":"result","payload":{{"verdict":"{verdict}","summary":"Reviewed the controlled assertion","evidence":["{evidence}"],"revisions":[]}}}}"#
            );
        }
    }
}
