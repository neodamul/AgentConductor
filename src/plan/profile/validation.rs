use super::*;
impl AgentProfile {
    pub fn validate_model(&self, model: &str) -> Result<(), String> {
        if model.is_empty()
            || model.starts_with('-')
            || model.chars().any(char::is_whitespace)
            || !self
                .catalog
                .as_ref()
                .is_some_and(|c| !c.source.is_empty() && c.models.iter().any(|m| m == model))
        {
            return Err("model must belong to an observed CLI catalog".into());
        }
        if self.parser == crate::protocol::Provider::Custom
            && !self.args.iter().any(|a| a.contains("{model}"))
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
        if self.agent_settings.is_some() {
            crate::agent_settings::settings_for(self.parser, self.agent_settings.as_ref())?;
        }
        if self.protocol.is_some() {
            if !custom {
                return Err("the acio protocol requires a custom profile".into());
            }
            if !self.event_types.is_empty()
                || self.session_id_pointer.is_some()
                || self.result_pointer.is_some()
                || self.error_condition.is_some()
                || self.denial_condition.is_some()
                || self.on_invalid_json != InvalidJson::Fail
            {
                return Err("the acio protocol defines its own event mapping; remove the custom event mappings".into());
            }
        }
        if !custom
            && (self.model_discovery.is_some()
                || !self.execution_options.is_empty()
                || self.result_pointer.is_some()
                || self.error_condition.is_some()
                || self.denial_condition.is_some()
                || self.on_invalid_json != InvalidJson::Fail)
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
        if let Some(denial) = &self.denial_condition {
            let message_match =
                denial.equals.is_some() != denial.contains.as_ref().is_some_and(|c| !c.is_empty());
            if !message_match
                || denial
                    .equals
                    .as_ref()
                    .is_some_and(serde_json::Value::is_null)
                || !denial.pointer.starts_with('/')
                || !denial
                    .tool_pointer
                    .iter()
                    .chain(&denial.message_pointer)
                    .all(|p| pointer(p))
            {
                return Err("custom denial condition needs a JSON Pointer and exactly one of equals or contains".into());
            }
        }
        Ok(())
    }

    /// GitHub #341: a non-blocking hint when automatic approval is declared but nothing could show that a tool
    /// was refused, which is how a missing approval flag goes unnoticed.
    pub fn configuration_hint(&self) -> Option<&'static str> {
        (self.parser == crate::protocol::Provider::Custom
            && self.permission_mode == "operator_auto_approve"
            && self.protocol.is_none()
            && self.event_types.is_empty()
            && self.error_condition.is_none()
            && self.denial_condition.is_none())
        .then_some("startup.auto_approve_unconfigured")
    }

    pub fn quota_key(&self) -> &str {
        self.quota_group.as_deref().unwrap_or(&self.executable)
    }
}
