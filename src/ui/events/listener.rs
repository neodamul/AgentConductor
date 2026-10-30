use super::{Event, EventBus};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, mpsc},
    thread,
    time::{Duration, SystemTime},
};

const CHECK_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, Debug, Eq, PartialEq)]
struct Fingerprint {
    length: u64,
    modified: Option<SystemTime>,
}

pub struct RunChangeListener {
    stop: mpsc::Sender<()>,
    thread: Option<thread::JoinHandle<()>>,
}

impl RunChangeListener {
    pub fn spawn(path: PathBuf, bus: Arc<EventBus>) -> Result<Self, String> {
        let observed = fingerprint(&path);
        let (stop, stopped) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("agentconductor-run-listener".into())
            .spawn(move || listen_report(path, bus, stopped, observed))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }

    pub fn spawn_startup(
        startup_path: PathBuf,
        resolve_run: impl Fn() -> Option<PathBuf> + Send + 'static,
        bus: Arc<EventBus>,
    ) -> Result<Self, String> {
        let observed_startup = fingerprint(&startup_path);
        let report = resolve_run();
        let observed_report = report.as_deref().and_then(fingerprint);
        let (stop, stopped) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("agentconductor-workspace-listener".into())
            .spawn(move || {
                listen_startup(
                    startup_path,
                    report,
                    resolve_run,
                    bus,
                    stopped,
                    observed_startup,
                    observed_report,
                )
            })
            .map_err(|error| error.to_string())?;
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for RunChangeListener {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn listen_report(
    path: PathBuf,
    bus: Arc<EventBus>,
    stopped: mpsc::Receiver<()>,
    mut observed: Option<Fingerprint>,
) {
    loop {
        match stopped.recv_timeout(CHECK_INTERVAL) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let current = fingerprint(&path);
        if current != observed {
            observed = current;
            bus.publish(None, Event::RunChanged);
        }
    }
}

fn listen_startup(
    startup_path: PathBuf,
    mut report_path: Option<PathBuf>,
    resolve_run: impl Fn() -> Option<PathBuf>,
    bus: Arc<EventBus>,
    stopped: mpsc::Receiver<()>,
    mut observed_startup: Option<Fingerprint>,
    mut observed_report: Option<Fingerprint>,
) {
    loop {
        match stopped.recv_timeout(CHECK_INTERVAL) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let current_startup = fingerprint(&startup_path);
        if current_startup != observed_startup {
            observed_startup = current_startup;
            bus.publish(None, Event::WorkspaceChanged);
            report_path = resolve_run();
            observed_report = report_path.as_deref().and_then(fingerprint);
        }
        let current_report = report_path.as_deref().and_then(fingerprint);
        if current_report != observed_report {
            observed_report = current_report;
            bus.publish(None, Event::RunChanged);
        }
    }
}

fn fingerprint(path: &std::path::Path) -> Option<Fingerprint> {
    let metadata = fs::metadata(path).ok()?;
    Some(Fingerprint {
        length: metadata.len(),
        modified: metadata.modified().ok(),
    })
}
