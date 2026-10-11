//! DD-55: operator role rules that constrain which agent the planner may assign.
use crate::plan::{AgentProfile, Task};
use std::collections::BTreeMap;

/// Role → ordered registered agent IDs (first preferred). Planning is the coordinator.
pub type RoleRules = BTreeMap<String, Vec<String>>;

pub const ROLES: [&str; 5] = [
    "verification",
    "development",
    "documentation",
    "knowledge",
    "testing",
];

pub fn validate(rules: &RoleRules, profiles: &[AgentProfile]) -> Result<(), String> {
    for (role, agents) in rules {
        if !ROLES.contains(&role.as_str()) {
            return Err(format!(
                "unknown role {role}; use one of {}",
                ROLES.join(", ")
            ));
        }
        if agents.len() > 5 {
            return Err(format!("role {role} lists more than five agents"));
        }
        if let Some(agent) = agents
            .iter()
            .find(|a| !profiles.iter().any(|p| &p.id == *a))
        {
            return Err(format!("role {role} names unregistered agent {agent}"));
        }
    }
    Ok(())
}

/// Planning-prompt section; empty when no rules are configured.
pub fn prompt(rules: &RoleRules) -> String {
    let lines: Vec<String> = rules
        .iter()
        .filter(|(_, agents)| !agents.is_empty())
        .map(|(role, agents)| format!("- {role}: {}", agents.join(", ")))
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    format!(
        "\nRole rules (set by the operator; follow them exactly):\n{}\nGive every task a \"role\" field with one of these roles and set \"agent\" to an agent listed for that role, preferring the first.\n",
        lines.join("\n")
    )
}

/// Rejects a generated plan whose tasks lack a role or use an agent outside it.
pub fn check(rules: &RoleRules, tasks: &[Task]) -> Result<(), String> {
    if rules.values().all(Vec::is_empty) {
        return Ok(());
    }
    for task in tasks {
        let role = task.role.as_deref().ok_or_else(|| {
            format!(
                "coordinator gave task {} no role despite role rules",
                task.id
            )
        })?;
        let allowed = rules
            .get(role)
            .filter(|a| !a.is_empty())
            .ok_or_else(|| format!("task {} uses role {role}, which has no rule", task.id))?;
        if !allowed.contains(&task.agent) {
            return Err(format!(
                "task {} assigns {} but role {role} allows only {}",
                task.id,
                task.agent,
                allowed.join(", ")
            ));
        }
    }
    Ok(())
}
