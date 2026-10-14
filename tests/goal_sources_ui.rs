//! User request 2026-10-14: the coordinator sits beside the workspace name and the project folder
//! above the goal; MCP lookups show progress, the preview window opens before the lookup ends,
//! and goal references are tags with a hover summary that stay plain, deletable text.
const HTML: &str = include_str!("../src/ui/startup.html");
const MCP_HTML: &str = include_str!("../src/ui/mcp.html");
const MCP_JS: &str = include_str!("../src/ui/mcp.js");

#[test]
fn coordinator_beside_the_name_and_folder_above_the_goal() {
    let row = &HTML[HTML.find("name-coordinator-row").unwrap()..];
    let row = &row[..row.find("start-repository").unwrap()];
    assert!(row.contains("id=\"start-name\"") && row.contains("id=\"start-coordinator\""));
    assert!(
        HTML.find("id=\"start-repository\"").unwrap() < HTML.find("id=\"start-goal\"").unwrap()
    );
}

#[test]
fn the_preview_window_opens_first_and_shows_the_lookup() {
    assert!(MCP_HTML.contains("id=\"start-mcp-loading\""));
    let click = &MCP_JS[MCP_JS.find("q('mcp-preview').onclick").unwrap()..];
    let opens = click.find("q('mcp-review').showModal()").unwrap();
    let asks = click.find("action({kind:'mcp_preview'})").unwrap();
    assert!(
        opens < asks,
        "the window opens before the lookup is requested"
    );
    assert!(MCP_JS.contains("acMessage('mcp.lookup_running')"));
    assert!(MCP_JS.contains("if(!q('mcp-review').open)q('mcp-review').showModal();"));
}

#[test]
fn goal_references_are_tags_over_plain_text_with_a_hover_summary() {
    assert!(HTML.contains("id=\"start-goal-tags\"") && HTML.contains("id=\"start-goal-field\""));
    // The textarea stays the only editor, so tags delete like text.
    assert!(HTML.contains("<textarea id=\"start-goal\""));
    assert!(MCP_JS.contains("function goalTags()") && MCP_JS.contains("function tagTitle(ref)"));
    assert!(MCP_JS.contains("state()?.source_notes?.[ref]"));
}
