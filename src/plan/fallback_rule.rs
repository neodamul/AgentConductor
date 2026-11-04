//! #409: a reviewed task policy overrides a shared profile without mutating it.
use super::{AgentProfile, Fallback, Task};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FallbackRule {
    Disabled(bool),
    Agent(Fallback),
}
impl FallbackRule {
    pub fn validate(&self, source: &AgentProfile, profiles: &[AgentProfile]) -> Result<(), String> {
        match self {
            Self::Disabled(false) => Ok(()),
            Self::Disabled(true) => {
                Err("fallback must be false or an explicit agent policy".into())
            }
            Self::Agent(policy) => validate_policy(source, policy, profiles),
        }
    }
}
fn validate_policy(
    source: &AgentProfile,
    policy: &Fallback,
    profiles: &[AgentProfile],
) -> Result<(), String> {
    let target = profiles
        .iter()
        .find(|p| p.id == policy.agent)
        .ok_or("unknown fallback agent")?;
    if source.id == target.id || source.quota_key() == target.quota_key() {
        return Err("fallback must use a different agent and account quota group".into());
    }
    if let Some(model) = &policy.model {
        target.validate_model(model)?;
    }
    Ok(())
}
impl Task {
    pub fn fallback_for<'a>(
        &'a self,
        profiles: &'a [AgentProfile],
    ) -> Result<Option<&'a Fallback>, String> {
        let source = profiles
            .iter()
            .find(|p| p.id == self.agent)
            .ok_or("unknown task agent")?;
        let policy = match &self.fallback {
            None => source.fallback.as_ref(),
            Some(FallbackRule::Disabled(false)) => None,
            Some(FallbackRule::Disabled(true)) => {
                return Err("fallback must be false or an explicit agent policy".into());
            }
            Some(FallbackRule::Agent(policy)) => Some(policy),
        };
        if let Some(policy) = policy {
            validate_policy(source, policy, profiles)?;
        }
        Ok(policy)
    }
}
