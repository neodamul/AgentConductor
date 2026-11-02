use super::*;

impl AgentProfile {
    pub fn validate_model(&self, model: &str) -> Result<(), String> {
        if model.is_empty()
            || model.starts_with('-')
            || model.chars().any(char::is_whitespace)
            || !self.catalog.as_ref().is_some_and(|c| {
                !c.source.is_empty() && c.models.iter().any(|candidate| candidate == model)
            })
        {
            return Err("model must belong to an observed CLI catalog".into());
        }
        if self.parser == crate::protocol::Provider::Custom
            && !self.args.iter().any(|arg| arg.contains("{model}"))
            && !self
                .read_only_args
                .as_ref()
                .is_some_and(|args| args.iter().any(|arg| arg.contains("{model}")))
            && !self
                .execution_options
                .iter()
                .any(|option| option.name == "model")
        {
            return Err("custom model selection requires a {model} argv placeholder".into());
        }
        Ok(())
    }
    pub fn validate_custom_configuration(&self) -> Result<(), String> {
        let custom = self.parser == crate::protocol::Provider::Custom;
        let pointer = |value: &str| value.is_empty() || value.starts_with('/');
        if let Some(model) = &self.coordinator_model {
            self.validate_model(model)?;
        }
        if let Some(args) = &self.read_only_args
            && (!custom
                || args.is_empty()
                || args.len() > 64
                || args == &self.args
                || args
                    .iter()
                    .any(|arg| arg.is_empty() || arg.len() > 4096 || arg.contains('\0')))
        {
            return Err("invalid custom read-only arguments".into());
        }
        if self.protocol.is_some() {
            if !custom {
                return Err("the acio protocol requires a custom profile".into());
            }
            if !self.event_types.is_empty()
                || self.session_id_pointer.is_some()
                || self.result_pointer.is_some()
                || self.error_condition.is_some()
            {
                return Err("the acio protocol defines its own event mapping; remove the custom event mappings".into());
            }
        }
        if !custom
            && (self.model_discovery.is_some()
                || !self.execution_options.is_empty()
                || self.result_pointer.is_some()
                || self.error_condition.is_some())
        {
            return Err("custom CLI configuration requires a custom profile".into());
        }
        if let Some(discovery) = &self.model_discovery
            && (discovery.args.is_empty()
                || discovery.args.len() > 64
                || !(1..=300).contains(&discovery.timeout_seconds)
                || discovery
                    .args
                    .iter()
                    .any(|arg| arg.len() > 4096 || arg.contains('\0'))
                || !pointer(&discovery.models_pointer)
                || !pointer(&discovery.model_pointer))
        {
            return Err("invalid custom model discovery configuration".into());
        }
        let mut names = std::collections::HashSet::new();
        for option in &self.execution_options {
            if option.name.trim().is_empty()
                || !names.insert(&option.name)
                || option.args.is_empty()
                || option.args.len() > 32
                || option
                    .values
                    .iter()
                    .any(|value| value.is_empty() || value.starts_with('-'))
                || option.default.as_ref().is_some_and(|value| {
                    !option.values.is_empty() && !option.values.contains(value)
                })
                || option.name == "model" && !option.values.is_empty()
                || option.model_values.iter().any(|(model, values)| {
                    self.catalog
                        .as_ref()
                        .is_none_or(|catalog| !catalog.models.contains(model))
                        || values.iter().any(|value| !option.values.contains(value))
                })
            {
                return Err(format!("invalid execution option group {}", option.name));
            }
        }
        if self
            .execution_options
            .iter()
            .any(|option| option.name == "model")
            && self.args.iter().any(|arg| arg.contains("{model}"))
        {
            return Err(
                "custom model selection must use either its option group or an argv placeholder"
                    .into(),
            );
        }
        for value in self
            .result_pointer
            .iter()
            .chain(self.error_condition.iter().map(|c| &c.pointer))
        {
            if !pointer(value) {
                return Err("invalid custom result JSON Pointer".into());
            }
        }
        if self
            .error_condition
            .as_ref()
            .is_some_and(|condition| condition.equals.is_null())
        {
            return Err("custom error condition needs a non-null match value".into());
        }
        crate::agent_settings::settings_for(self.parser, self.agent_settings.as_ref())?;
        Ok(())
    }
    pub fn quota_key(&self) -> &str {
        self.quota_group.as_deref().unwrap_or(&self.executable)
    }
}
