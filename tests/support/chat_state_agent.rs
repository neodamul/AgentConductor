use std::io::Read;
fn main() {
 let mut prompt=String::new();std::io::stdin().read_to_string(&mut prompt).unwrap();
 std::fs::write(std::env::var("AC_CHAT_CAPTURE").unwrap(),prompt).unwrap();
 std::thread::sleep(std::time::Duration::from_millis(500));
 println!("{}",r#"{"type":"result","is_error":false,"result":"{\"answer\":\"fresh answer\"}"}"#);
}
