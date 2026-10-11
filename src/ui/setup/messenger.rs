//! DD-79 / DD-81: messenger settings for the workspace page. Secrets are written to the
//! machine-local store and never returned.
use super::*;

impl Startup {
    /// `secrets` = [webhook URL, Slack bot token, relay URL, relay secret]; `None` keeps the
    /// saved value and an empty string removes it.
    pub(super) fn save_messenger(
        &self,
        saved: &mut Saved,
        settings: crate::messenger::Settings,
        secrets: [Option<String>; 4],
    ) -> Result<serde_json::Value, String> {
        if let Some(template) = settings
            .template
            .as_deref()
            .filter(|_| settings.kind == crate::messenger::Kind::Webhook)
        {
            let probe = crate::messenger::Settings {
                template: Some(template.into()),
                ..settings.clone()
            };
            crate::messenger::payload(&probe, &crate::messenger::sample())?;
        }
        if settings.approvers.len() > 32
            || settings
                .approvers
                .iter()
                .any(|a| a.is_empty() || a.len() > 128)
        {
            return Err("approvers must be 1–32 non-empty IDs".into());
        }
        let names = [None, Some("token"), Some("relay"), Some("relay_secret")];
        for (name, value) in names.into_iter().zip(secrets) {
            if let Some(value) = value {
                let value = Some(value.as_str()).filter(|v| !v.is_empty());
                crate::messenger::save_secret_value(&self.broker.dir, name, value)?;
            }
        }
        let mut next = saved.clone();
        next.config.messenger = settings;
        self.store(&next)?;
        *saved = next;
        Ok(crate::messenger::status(
            &self.broker.dir,
            &saved.config.messenger,
        ))
    }
}
