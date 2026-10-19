use super::AgentProfile;

impl AgentProfile {
    pub fn validate_model(&self, model: &str) -> Result<(), String> {
        if model.is_empty()
            || model.starts_with('-')
            || model.chars().any(char::is_whitespace)
            || !self.catalog.as_ref().is_some_and(|catalog| {
                !catalog.source.is_empty() && catalog.models.iter().any(|m| m == model)
            })
        {
            return Err("model must belong to an observed CLI catalog".into());
        }
        if self.parser == crate::protocol::Provider::Custom
            && !self.args.iter().any(|arg| arg.contains("{model}"))
        {
            return Err("custom model selection requires a {model} argv placeholder".into());
        }
        Ok(())
    }

    pub fn quota_key(&self) -> &str {
        self.quota_group.as_deref().unwrap_or(&self.executable)
    }
}
