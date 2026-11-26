use agentconductor::manual::render_chat;

#[test]
fn the_chat_renders_markdown_and_preserves_plain_text() {
    let html = render_chat(
        "# Results\n\n**Done** and *checked* with `cargo test`.\n\n1. First\n2. Second\n\n- Other\n\n> A note\n\nName | Value\n--- | ---\na | b\n\n```sh\necho ok\n```",
    );
    for expected in [
        "<h1>Results</h1>",
        "<strong>Done</strong>",
        "<em>checked</em>",
        "<code>cargo test</code>",
        "<ol>",
        "<li>Second</li>",
        "<ul>",
        "<blockquote>",
        "<td>b</td>",
        "<pre><code>echo ok</code></pre>",
    ] {
        assert!(html.contains(expected), "missing {expected} in {html}");
    }
    assert_eq!(
        render_chat("line one\nline two"),
        "<p>line one\nline two</p>"
    );
}

#[test]
fn model_text_cannot_load_resources_or_make_unsafe_links() {
    let html = render_chat(
        "<script>bad()</script>\n<img src=x onerror=bad()>\n\n![remote](https://example.com/img.png)\n![local](/etc/hosts)\n\n[bad](javascript:alert) [file](file:///etc/hosts) [relative](//example.com) [docs](https://example.com/a?x=1&y=2)",
    );
    assert!(!html.contains("<script") && !html.contains("<img"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("href=\"https://example.com/a?x=1&amp;y=2\""));
    assert_eq!(html.matches("href=").count(), 1);
}

#[test]
fn native_room_formats_a_bounded_batch_of_narrative_text() {
    use agentconductor::ui::Room;
    let root = std::env::temp_dir().join(format!("ac-room-markdown-{}", std::process::id()));
    let room = Room::startup(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let request = serde_json::json!({"kind":"render_markdown","texts":["# Heading\n\n**Done**\nline two", "<img src=x onerror=bad()> [ignored](javascript:bad)"]});
    let result = room
        .dispatch(&serde_json::to_vec(&request).unwrap())
        .unwrap();
    assert!(
        result["html"][0]
            .as_str()
            .unwrap()
            .contains("<strong>Done</strong>\nline two")
    );
    let hostile = result["html"][1].as_str().unwrap();
    assert!(hostile.contains("&lt;img"));
    assert!(!hostile.contains("<img") && !hostile.contains("href="));
    assert!(
        room.dispatch(br#"{"kind":"render_markdown","texts":[123]}"#)
            .is_err()
    );
    let oversized = serde_json::json!({"kind":"render_markdown","texts":vec!["hello";17]});
    assert!(
        room.dispatch(&serde_json::to_vec(&oversized).unwrap())
            .is_err()
    );
    assert!(
        room.dispatch(br#"{"kind":"open_chat_link","url":"javascript:bad()"}"#)
            .is_err()
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
