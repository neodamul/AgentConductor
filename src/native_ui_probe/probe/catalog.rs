use super::{Pending, Probe, workspace::profile_tools};

impl Probe {
    pub(in crate::native_ui_probe) fn save_catalog(
        &mut self,
        agent: &str,
        catalog: &crate::plan::ModelCatalog,
    ) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            self.error = Some("Workspace revision is unavailable".into());
            return;
        };
        let Some(revision) = snapshot["revision"].as_str() else {
            self.error = Some("Workspace revision is unavailable".into());
            return;
        };
        let Some(profile) = self.profiles.iter().find(|profile| profile["id"] == agent) else {
            self.error = Some("Unknown agent profile".into());
            return;
        };
        if profile["parser"] != "custom" {
            self.error = Some("Manual model catalogs are limited to custom CLI profiles".into());
            return;
        }
        let packet = profile_tools::catalog_request(revision, agent, catalog);
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&packet))
        {
            Ok(()) => {
                self.pending = Some(Pending::CatalogSave);
                self.error = None;
                self.profile_editor.set_catalog_error(None);
            }
            Err(error) => self.profile_editor.set_catalog_error(Some(error)),
        }
    }
}
