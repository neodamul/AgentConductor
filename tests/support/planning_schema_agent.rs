use std::io::Read;
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let i = args
        .iter()
        .position(|a| a == "--json-schema")
        .expect("missing enforced output schema");
    let schema = &args[i + 1];
    assert!(schema.contains("\"phases\"") && schema.contains("\"type\":\"array\""));
    let mut prompt = String::new();
    std::io::stdin().read_to_string(&mut prompt).unwrap();
    assert!(prompt.contains("Required response JSON Schema"));
    println!(
        "{}",
        r#"{"type":"result","is_error":false,"structured_output":{"slug":"schema_check","analysis":"Analysis","phases":[{"id":"01-core","name":"Core","tasks":[{"id":"a","agent":"claude","prompt":"Check","paths":["one.txt"],"depends_on":[],"test_command":["git","diff","--check"],"role":null,"model":null}]}]}}"#
    );
}
