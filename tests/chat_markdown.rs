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
