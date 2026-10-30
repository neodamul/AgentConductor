//! GitHub #262 (DD-205): the facts line of a task (type, effort) is placed in the summary grid explicitly; auto-placement
//! put it in the 22px arrow column, where it became a column one character wide.
#[test]
fn the_task_facts_line_is_placed_under_the_text_column() {
    let css = std::fs::read_to_string("src/ui/startup.css").unwrap();
    let rule = css
        .split('}')
        .find(|rule| rule.contains("summary .task-profile") && !rule.contains("task-summary"))
        .expect("a rule for the facts line inside the task summary");
    let rule: String = rule.split_whitespace().collect();
    assert!(rule.contains("grid-column:2/-1"), "{rule}");
    assert!(rule.contains("grid-row:3"), "{rule}");
}
