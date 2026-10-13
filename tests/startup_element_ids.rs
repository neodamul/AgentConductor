//! A startup.js line that touches a removed element throws while the page loads and stops
//! every later setup line (Escape, outside-click and more), so each q('id') must exist.
const SCRIPT: &str = include_str!("../src/ui/startup.js");
const PAGES: [&str; 3] = [
    include_str!("../src/ui/startup.html"),
    include_str!("../src/ui/mcp.html"),
    include_str!("../src/ui/setup/document.rs"),
];

#[test]
fn every_element_the_startup_script_looks_up_exists() {
    let mut missing = vec![];
    for part in SCRIPT.split("q('").skip(1) {
        let Some(end) = part.find("')") else { continue };
        let id = &part[..end];
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            continue;
        }
        let markup = format!("id=\"start-{id}\"");
        let escaped = format!("id=\\\"start-{id}\\\"");
        if !PAGES
            .iter()
            .any(|page| page.contains(&markup) || page.contains(&escaped))
        {
            missing.push(id.to_string());
        }
    }
    missing.sort();
    missing.dedup();
    assert!(missing.is_empty(), "missing elements: {missing:?}");
}
