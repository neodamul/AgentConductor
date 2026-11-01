use super::Job;
use crate::{process::record_owner, state::Store};
use std::{
    io,
    path::PathBuf,
    process::{Child, ExitStatus},
    sync::{Arc, Mutex},
    thread,
};

pub(super) fn start(
    child: Arc<Mutex<Child>>,
    id: String,
    job_slot: Arc<Mutex<Option<Job>>>,
    dir: PathBuf,
) -> io::Result<()> {
    thread::Builder::new()
        .name("native-ui-job-monitor".into())
        .spawn(move || {
            let status = child
                .lock()
                .map_err(|_| io::Error::other("child process lock poisoned"))
                .and_then(|mut child| child.wait());
            complete(status, &id, &job_slot, &dir);
        })?;
    Ok(())
}

fn complete(
    status: io::Result<ExitStatus>,
    id: &str,
    job_slot: &Arc<Mutex<Option<Job>>>,
    dir: &std::path::Path,
) {
    let Ok(mut slot) = job_slot.lock() else {
        return;
    };
    let Some(job) = slot.as_mut().filter(|job| job.id == id) else {
        return;
    };
    job.owner.phase = "terminal".into();
    let mut errors = Vec::new();
    match status {
        Ok(status) => {
            job.exit = status.code();
            job.owner.exit_code = status.code();
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                job.owner.termination_signal = status.signal();
            }
        }
        Err(error) => errors.push(format!("could not wait for coordinator process: {error}")),
    }
    let (logs, owner, label, exit) = (
        job.logs.clone(),
        job.owner.clone(),
        job.label.clone(),
        job.exit,
    );
    drop(slot);

    if let Err(error) = record_owner(&logs, &owner) {
        errors.push(format!(
            "could not save coordinator process status: {error}"
        ));
    }
    let event = crate::ui::messages::render(
        "native.ui_action_finished_with_exit_signal",
        &[
            ("value1", &label),
            ("code", &format!("{:?}", exit)),
            ("value2", &format!("{:?}", owner.termination_signal)),
        ],
    );
    let result = serde_json::json!({
        "id":id,
        "kind":label,
        "exit_code":exit,
        "termination_signal":owner.termination_signal
    });
    if let Err(error) = Store::open(dir.join("state.sqlite"))
        .and_then(|mut store| store.record("ui_action_result", &result, &event))
    {
        errors.push(format!("could not save coordinator action result: {error}"));
    }
    if let Ok(mut slot) = job_slot.lock()
        && let Some(job) = slot.as_mut().filter(|job| job.id == id)
    {
        job.finished = true;
        job.audit_error = (!errors.is_empty()).then(|| errors.join("; "));
    }
}
