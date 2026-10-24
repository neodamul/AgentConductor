use serde_json::Value;

pub fn suggested(profiles: &[Value]) -> crate::roles::RoleRules {
    let agents = |parsers: &[&str]| {
        parsers
            .iter()
            .flat_map(|parser| {
                profiles.iter().filter(move |profile| {
                    profile["parser"].as_str().unwrap_or("custom") == *parser
                })
            })
            .filter_map(|profile| profile["id"].as_str())
            .take(2)
            .map(|agent| crate::roles::RoleAgent {
                agent: agent.into(),
                model: None,
            })
            .collect::<Vec<_>>()
    };
    let mut rules = crate::roles::RoleRules::new();
    for (role, parsers) in [
        ("verification", &["claude", "codex"][..]),
        ("development", &["codex", "claude"][..]),
        ("documentation", &["codex", "claude"][..]),
        ("knowledge", &["codex", "claude"][..]),
        ("testing", &["opencode", "custom", "claude"][..]),
    ] {
        let choices = agents(parsers);
        if !choices.is_empty() {
            rules.insert(role.into(), choices);
        }
    }
    rules
}
