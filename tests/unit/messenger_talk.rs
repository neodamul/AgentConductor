use super::*;

fn settings() -> Settings {
    Settings {
        enabled: true,
        conversation: true,
        talkers: vec!["ann".into()],
        ..Default::default()
    }
}

fn msg(ts: &str, sender: &str, text: &str) -> Incoming {
    Incoming {
        id: ts.into(),
        sender: sender.into(),
        ts: ts.into(),
        thread: None,
        text: text.into(),
    }
}

fn run(inbox: &mut Inbox, messages: Vec<Incoming>, now: u64) -> Vec<String> {
    let mut said = vec![];
    apply(&settings(), inbox, messages, now, &mut |t| said.push(t));
    said
}

#[test]
fn queued_and_rejected_messages_are_audited_and_strangers_are_not() {
    let mut inbox = Inbox {
        started: true,
        ..Default::default()
    };
    let audit = apply(
        &settings(),
        &mut inbox,
        vec![
            msg("1.0", "mallory", "hi"),
            msg("2.0", "ann", "q"),
            msg("3.0", "ann", &"x".repeat(2001)),
        ],
        10,
        &mut |_| {},
    );
    assert_eq!(
        audit,
        vec![
            ("2.0".to_string(), "ann".to_string(), "queued"),
            ("3.0".to_string(), "ann".to_string(), "rejected")
        ]
    );
}

#[test]
fn the_first_poll_skips_history_and_later_ones_answer_in_order() {
    let mut inbox = Inbox::default();
    assert!(run(&mut inbox, vec![msg("1.0", "ann", "old question")], 10).is_empty());
    assert!(inbox.queue.is_empty() && inbox.cursor == "1.0" && inbox.started);
    // An empty first poll still counts as started, so the next message is not skipped.
    let mut empty = Inbox::default();
    run(&mut empty, vec![], 10);
    run(&mut empty, vec![msg("2.0", "ann", "hello")], 11);
    assert_eq!(empty.queue.len(), 1);
    // The second message waits behind the first and the sender is told its place.
    let said = run(
        &mut inbox,
        vec![msg("2.0", "ann", "one"), msg("3.0", "ann", "two")],
        12,
    );
    assert_eq!(inbox.queue.len(), 2);
    assert_eq!(
        said.len(),
        1,
        "only the second needs an acknowledgement: {said:?}"
    );
    assert_eq!(inbox.cursor, "3.0");
    // A channel that repeats old messages after the cursor moved is not answered twice.
    let again = run(
        &mut inbox,
        vec![msg("2.0", "ann", "one"), msg("3.0", "ann", "two")],
        13,
    );
    assert!(again.is_empty() && inbox.queue.len() == 2);
}

#[test]
fn rejections_are_answered_and_strangers_stay_silent() {
    let mut inbox = Inbox {
        started: true,
        ..Default::default()
    };
    let said = run(
        &mut inbox,
        vec![
            msg("1.0", "mallory", "hi"),
            msg("2.0", "ann", &"x".repeat(2001)),
        ],
        10,
    );
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("2,000"), "{}", said[0]);
    assert!(inbox.queue.is_empty());
}

#[test]
fn the_interval_is_short_only_while_a_conversation_is_going_on() {
    let mut inbox = Inbox::default();
    assert_eq!(interval(&inbox, 10_000), IDLE);
    inbox.last_talk = 9_900;
    assert_eq!(interval(&inbox, 10_000), ACTIVE);
    inbox.last_talk = 0;
    inbox.queue.push(conversation::Queued {
        id: "a".into(),
        sender: "ann".into(),
        text: "q".into(),
    });
    assert_eq!(interval(&inbox, 10_000), ACTIVE);
}

#[test]
fn the_inbox_survives_a_restart() {
    let root = std::env::temp_dir().join(format!("ac-talk-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    assert!(!load(&root).started, "a missing file is a fresh inbox");
    let mut inbox = Inbox::default();
    run(&mut inbox, vec![], 1);
    save(&root, &inbox).unwrap();
    assert!(load(&root).started);
    std::fs::remove_dir_all(root).unwrap();
}
