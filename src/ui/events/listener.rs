use super::{Event, EventBus};
#[path = "listener/scope.rs"]
mod scope;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, mpsc},
    thread,
    time::{Duration, SystemTime},
};

#[cfg(test)]
#[path = "../../../tests/support/persisted_state_event.rs"]
mod tests;

pub(super) const RECOVERY_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileStamp {
    length: u64,
    modified: Option<SystemTime>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Fingerprint {
    primary: Option<FileStamp>,
    database: Option<FileStamp>,
    wal: Option<FileStamp>,
    scope: Vec<(PathBuf, FileStamp)>,
}

pub struct RunChangeListener {
    stop: mpsc::Sender<()>,
    thread: Option<thread::JoinHandle<()>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupWatch {
    pub startup_path: PathBuf,
    pub report_path: Option<PathBuf>,
}

impl RunChangeListener {
    pub fn spawn(path: PathBuf, bus: Arc<EventBus>) -> Result<Self, String> {
        Self::spawn_with_interval(path, bus, RECOVERY_INTERVAL)
    }

    pub(super) fn spawn_with_interval(
        path: PathBuf,
        bus: Arc<EventBus>,
        interval: Duration,
    ) -> Result<Self, String> {
        let observed = fingerprint(&path);
        let (stop, stopped) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("agentconductor-run-listener".into())
            .spawn(move || listen_report(path, bus, stopped, observed, interval))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }

    pub fn spawn_startup(
        resolve_watch: impl Fn() -> Option<StartupWatch> + Send + 'static,
        bus: Arc<EventBus>,
    ) -> Result<Self, String> {
        Self::spawn_startup_with_interval(resolve_watch, bus, RECOVERY_INTERVAL)
    }

    pub(super) fn spawn_startup_with_interval(
        resolve_watch: impl Fn() -> Option<StartupWatch> + Send + 'static,
        bus: Arc<EventBus>,
        interval: Duration,
    ) -> Result<Self, String> {
        let observed = resolve_watch();
        let observed_startup = observed
            .as_ref()
            .and_then(|watch| fingerprint(&watch.startup_path));
        let observed_report = observed
            .as_ref()
            .and_then(|watch| watch.report_path.as_deref())
            .and_then(fingerprint);
        let (stop, stopped) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("agentconductor-workspace-listener".into())
            .spawn(move || {
                listen_startup(
                    observed,
                    resolve_watch,
                    bus,
                    stopped,
                    observed_startup,
                    observed_report,
                    interval,
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
    interval: Duration,
) {
    loop {
        match stopped.recv_timeout(interval) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let current = fingerprint(&path);
        let changed = current != observed;
        if changed {
            observed = current;
            bus.publish(None, Event::RunChanged);
        }
    }
}

fn listen_startup(
    mut watch: Option<StartupWatch>,
    resolve_watch: impl Fn() -> Option<StartupWatch>,
    bus: Arc<EventBus>,
    stopped: mpsc::Receiver<()>,
    mut observed_startup: Option<Fingerprint>,
    mut observed_report: Option<Fingerprint>,
    interval: Duration,
) {
    loop {
        match stopped.recv_timeout(interval) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let current_watch = resolve_watch();
        let path_changed = current_watch.as_ref().map(|w| &w.startup_path)
            != watch.as_ref().map(|w| &w.startup_path);
        let current_startup = current_watch
            .as_ref()
            .and_then(|current| fingerprint(&current.startup_path));
        let startup_changed = path_changed || current_startup != observed_startup;
        if startup_changed {
            observed_startup = current_startup;
            bus.publish(None, Event::WorkspaceChanged);
            observed_report = current_watch
                .as_ref()
                .and_then(|current| current.report_path.as_deref())
                .and_then(fingerprint);
        }
        watch = current_watch;
        let current_report = watch
            .as_ref()
            .and_then(|current| current.report_path.as_deref())
            .and_then(fingerprint);
        let report_changed = current_report != observed_report;
        if report_changed {
            observed_report = current_report;
            bus.publish(None, Event::RunChanged);
        }
    }
}

fn fingerprint(path: &std::path::Path) -> Option<Fingerprint> {
    let parent = path.parent()?;
    let stamp = |path: &std::path::Path| {
        fs::metadata(path).ok().map(|metadata| FileStamp {
            length: metadata.len(),
            modified: metadata.modified().ok(),
        })
    };
    let primary = stamp(path);
    let database = stamp(&parent.join("state.sqlite"));
    let wal = stamp(&parent.join("state.sqlite-wal"));
    let scope: Vec<_> = scope::declared_paths(path)
        .into_iter()
        .filter_map(|path| scope::stamp(&path).map(|stamp| (path, stamp)))
        .collect();
    if primary.is_none() && database.is_none() && wal.is_none() && scope.is_empty() {
        return None;
    }
    Some(Fingerprint {
        primary,
        database,
        wal,
        scope,
    })
}
