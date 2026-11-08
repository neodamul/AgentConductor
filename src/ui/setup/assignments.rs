//! GitHub #564: saving role rules moves the reviewed plan's tasks onto the agents the rules now allow.
use super::*;

/// Rewrites the ready plan in place; an error means the plan cannot follow the rules and must be regenerated.
pub(super) fn follow_roles(proposal: &Proposal, config: &Config) -> Result<(), String> {
    let path = proposal.directory.join("review.json");
    let mut plan: Plan = read(&path)?;
    crate::roles::reassign(&config.roles, &config.profiles, &mut plan.tasks)?;
    plan.agents = config.profiles.clone();
    plan.validate()?;
    write(&path, &plan)
}
