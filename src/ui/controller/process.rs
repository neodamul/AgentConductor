use super::{Controller, Job};
use crate::{process::record_owner, state::Store};

pub(super) fn breakaway_denied(error: &std::io::Error) -> bool {
    #[cfg(windows)]
    {
        error.raw_os_error() == Some(5)
    }
    #[cfg(not(windows))]
    {
        let _ = error;
        false
    }
}

impl Controller {
    pub(super) fn poll_unmonitored(&self, job: &mut Job) -> Result<(), String> {
        if let Some(status) = job
            .child
            .lock()
            .map_err(|_| "UI child process lock is poisoned")?
            .try_wait()
            .map_err(|error| error.to_string())?
        {
            job.exit = status.code();
            job.finished = true;
            job.owner.phase = "terminal".into();
            job.owner.exit_code = status.code();
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                job.owner.termination_signal = status.signal();
            }
            record_owner(&job.logs, &job.owner)?;
            Store::open(self.dir.join("state.sqlite"))?.record(
                "ui_action_result",
                &serde_json::json!({"id":job.id,"kind":job.label,"exit_code":job.exit,"termination_signal":job.owner.termination_signal}),
                &crate::ui::messages::render("native.ui_action_finished_with_exit_signal", &[("value1", &job.label), ("code", &format!("{:?}", job.exit)), ("value2", &format!("{:?}", job.owner.termination_signal))]),
            )?;
        }
        Ok(())
    }
}
