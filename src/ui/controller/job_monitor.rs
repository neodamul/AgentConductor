use super::Job;
use crate::{
    process::record_owner,
    state::Store,
    ui::events::{EventHub, UiEvent, UiSurface},
};
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
    events: EventHub,
    surface: UiSurface,
) -> io::Result<()> {
    thread::Builder::new()
        .name("native-ui-job-monitor".into())
        .spawn(move || {
            let status = child
                .lock()
                .map_err(|_| io::Error::other("child process lock poisoned"))
                .and_then(|mut child| child.wait());
            complete(status, &id, &job_slot, &dir);
            events.publish(UiEvent::invalidated(surface));
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
    let Some((logs, owner, label, exit, termination_signal)) =
        slot.as_mut().filter(|job| job.id == id).map(|job| {
            job.owner.phase = "terminal".into();
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
                Err(error) => eprintln!("could not wait for coordinator process: {error}"),
            }
            (
                job.logs.clone(),
                job.owner.clone(),
                job.label.clone(),
                job.exit,
                job.owner.termination_signal,
            )
        })
    else {
        return;
    };
    drop(slot);
    if let Err(error) = record_owner(&logs, &owner) {
        eprintln!("could not save coordinator process status: {error}");
    }
    let result = serde_json::json!({
        "id":id,
        "kind":label,
        "exit_code":exit,
        "termination_signal":termination_signal
    });
    let event = crate::ui::messages::render(
        "native.ui_action_finished_with_exit_signal",
        &[
            ("value1", &label),
            ("code", &format!("{:?}", exit)),
            ("value2", &format!("{:?}", termination_signal)),
        ],
    );
    match Store::open(dir.join("state.sqlite"))
        .and_then(|mut store| store.record("ui_action_result", &result, &event))
    {
        Ok(()) => {}
        Err(error) => eprintln!("could not save coordinator action result: {error}"),
    }
    if let Ok(mut slot) = job_slot.lock()
        && let Some(job) = slot.as_mut().filter(|job| job.id == id)
    {
        job.finished = true;
    }
}
