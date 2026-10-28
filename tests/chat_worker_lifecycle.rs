#[test]
fn chat_worker_is_classified_as_a_short_lived_maintenance_command() {
    let main = include_str!("../src/main.rs");
    assert!(
        main.contains("args.first().map(String::as_str) == Some(\"chat\")"),
        "chat workers must not wait for unrelated messenger shutdown work"
    );
}
