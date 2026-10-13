//! DD-128: native opening may recover/resume only an unchanged, quiescent run.
use super::*;
impl Controller {
    pub(super) fn automatic(&self, enabled: bool) -> Result<(), String> {
        let recovery = workflow::automatic_recover(&self.dir, enabled)?;
        if !recovery.resume {
            return Ok(());
        }
        let result = (|| {
            let report = workflow::load(&self.dir)?;
            let revision = workflow::revision(&report)?;
            let mut args = vec![
                "ui-action".into(),
                self.dir.to_string_lossy().into_owned(),
                revision.clone(),
            ];
            args.extend(Action::Resume {}.args(&report, &self.dir)?);
            let mut job = self.job.lock().map_err(|_| "UI recovery state poisoned")?;
            self.launch_locked("auto_resume", &args, &revision, &mut job)
        })();
        match result {
            Ok(_) => workflow::automatic_resume_result(&self.dir, true, None),
            Err(error) => workflow::automatic_resume_result(&self.dir, false, Some(&error)),
        }
    }
}
