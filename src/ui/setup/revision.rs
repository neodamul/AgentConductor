use super::*;

impl Startup {
    pub(super) fn revision(&self, saved: &Saved) -> Result<String, String> {
        self.state_revision(saved, true)
    }

    /// Chat checks the saved context but skips the repository HEAD subprocess.
    pub(super) fn chat_revision(&self, saved: &Saved) -> Result<String, String> {
        self.state_revision(saved, false)
    }

    fn state_revision(&self, saved: &Saved, include_head: bool) -> Result<String, String> {
        // Observations and scheduler progress cannot invalidate an unrelated edit.
        let mut context = serde_json::to_value(saved).map_err(|error| error.to_string())?;
        for key in [
            "quota_job",
            "metadata_queue",
            "metadata_input",
            "metadata_settled",
            "metadata_error",
        ] {
            context.as_object_mut().unwrap().remove(key);
        }
        if let Some(profiles) = context["config"]["profiles"].as_array_mut() {
            for profile in profiles {
                profile.as_object_mut().unwrap().remove("catalog");
            }
        }
        let mut bytes = serde_json::to_vec(&context).map_err(|error| error.to_string())?;
        if let Some(proposal) = &saved.proposal {
            let path = proposal.directory.join("review.json");
            if path.exists() {
                bytes.extend(super::bytes(&path)?);
            }
            if include_head {
                let project = proposal
                    .workspace
                    .as_deref()
                    .unwrap_or(&saved.config.repository);
                bytes.extend(self.heads.get(project).bytes());
            }
        }
        Ok(workflow::digest(&bytes))
    }
}
