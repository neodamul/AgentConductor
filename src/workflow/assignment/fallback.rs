use super::*;

/// Called only after JoinSet is empty. Reviewed policies never steer a live worker.
/// First agent on `source`'s reviewed fallback chain whose account is not restricted, with
/// the model its policy names. Agents may name each other; an agent already tried ends the
/// walk, so a chain of restricted accounts waits instead of looping.
pub(super) fn unrestricted_fallback(
    report: &Report,
    index: usize,
) -> Result<Option<(String, Option<String>)>, String> {
    let task = &report.plan.tasks[index];
    let mut profile = report
        .plan
        .agents
        .iter()
        .find(|agent| agent.id == task.agent)
        .ok_or("unknown task agent")?;
    let mut tried = std::collections::HashSet::from([profile.id.as_str()]);
    let mut next = task.fallback_for(&report.plan.agents)?;
    while let Some(policy) = next {
        let target = report
            .plan
            .agents
            .iter()
            .find(|agent| agent.id == policy.agent)
            .ok_or("unknown fallback agent")?;
        if !tried.insert(target.id.as_str()) {
            return Ok(None);
        }
        if !report.account_restrictions.contains_key(target.quota_key()) {
            return Ok(Some((target.id.clone(), policy.model.clone())));
        }
        profile = target;
        next = profile.fallback.as_ref();
    }
    Ok(None)
}
