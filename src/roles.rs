//! DD-55: operator role rules that constrain which agent (and model) the planner may assign.
use crate::plan::{AgentProfile, Task};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One allowed agent for a role, optionally pinned to an observed model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Entry")]
pub struct RoleAgent {
    pub agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Earlier rules stored plain agent IDs; both forms are accepted.
#[derive(Deserialize)]
#[serde(untagged)]
enum Entry {
    Name(String),
    Full {
        agent: String,
        #[serde(default)]
        model: Option<String>,
    },
}

impl From<Entry> for RoleAgent {
    fn from(entry: Entry) -> Self {
        match entry {
            Entry::Name(agent) => Self { agent, model: None },
            Entry::Full { agent, model } => Self {
                agent,
                model: model.filter(|m| !m.is_empty()),
            },
        }
    }
}

/// Role → ordered allowed agents (first preferred). Planning is the coordinator.
pub type RoleRules = BTreeMap<String, Vec<RoleAgent>>;

pub const ROLES: [&str; 5] = [
    "verification",
    "development",
    "documentation",
    "knowledge",
    "testing",
];

pub fn validate(rules: &RoleRules, profiles: &[AgentProfile]) -> Result<(), String> {
    for (role, entries) in rules {
        if !ROLES.contains(&role.as_str()) {
            return Err(format!(
                "unknown role {role}; use one of {}",
                ROLES.join(", ")
            ));
        }
        if entries.len() > 5 {
            return Err(format!("role {role} lists more than five agents"));
        }
        for entry in entries {
            let profile = profiles
                .iter()
                .find(|p| p.id == entry.agent)
                .ok_or_else(|| format!("role {role} names unregistered agent {}", entry.agent))?;
            if let Some(model) = &entry.model {
                profile
                    .validate_model(model)
                    .map_err(|e| format!("role {role}: {e}"))?;
            }
        }
    }
    Ok(())
}

/// Planning-prompt section; empty when no rules are configured.
pub fn prompt(rules: &RoleRules) -> String {
    let lines: Vec<String> = rules
        .iter()
        .filter(|(_, entries)| !entries.is_empty())
        .map(|(role, entries)| {
            let agents: Vec<String> = entries
                .iter()
                .map(|e| match &e.model {
                    Some(model) => format!("{} (model {model})", e.agent),
                    None => e.agent.clone(),
                })
                .collect();
            format!("- {role}: {}", agents.join(", "))
        })
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    format!(
        "\nRole rules (set by the operator; follow them exactly):\n{}\nGive every task a \"role\" field with one of these roles and set \"agent\" to an agent listed for that role, preferring the first.\n",
        lines.join("\n")
    )
}

/// Rejects tasks without a role or with an agent outside it, and applies pinned models.
pub fn apply(rules: &RoleRules, tasks: &mut [Task]) -> Result<(), String> {
    if rules.values().all(Vec::is_empty) {
        return Ok(());
    }
    for task in tasks {
        let role = task.role.clone().ok_or_else(|| {
            format!(
                "coordinator gave task {} no role despite role rules",
                task.id
            )
        })?;
        let entries = rules
            .get(&role)
            .filter(|a| !a.is_empty())
            .ok_or_else(|| format!("task {} uses role {role}, which has no rule", task.id))?;
        let entry = entries
            .iter()
            .find(|e| e.agent == task.agent)
            .ok_or_else(|| {
                let allowed: Vec<&str> = entries.iter().map(|e| e.agent.as_str()).collect();
                format!(
                    "task {} assigns {} but role {role} allows only {}",
                    task.id,
                    task.agent,
                    allowed.join(", ")
                )
            })?;
        if entry.model.is_some() {
            task.model = entry.model.clone();
        }
    }
    Ok(())
}
