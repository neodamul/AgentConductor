use std::io::{self, Read};
fn main() {
    let mut prompt = String::new();
    io::stdin().read_to_string(&mut prompt).unwrap();
    std::fs::write(std::env::var("AC_OPTIMIZER_SEEN").unwrap(), &prompt).unwrap();
    if prompt.contains("Windows acceptance") {
        println!(
            r#"{{"type":"result","payload":{{"optimized":"Goal for Windows","changes":["Used answers"],"questions":[]}}}}"#
        );
    } else {
        println!(
            r#"{{"type":"result","payload":{{"optimized":"Proposed goal","changes":["Clarified scope"],"questions":["Which environment?"]}}}}"#
        );
    }
}
