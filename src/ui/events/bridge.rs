use super::{EventPublisher, UiEvent, UiSurface};
use std::{
    collections::BTreeSet,
    fs,
    net::{SocketAddr, UdpSocket},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, SystemTime},
};

pub(super) const MIN_SCAN_DELAY: Duration = Duration::from_secs(1);
pub(super) const MAX_SCAN_DELAY: Duration = Duration::from_secs(30);
const MAX_SCOPE_PATHS: usize = 512;
const MAX_SCOPE_DEPTH: usize = 8;

pub(super) struct EventBridge {
    address: SocketAddr,
    token: String,
    stopped: Arc<AtomicBool>,
    shutdown: UdpSocket,
    watch: Arc<Mutex<Option<FileWatch>>>,
}

impl EventBridge {
    pub(super) fn new(publisher: EventPublisher) -> std::io::Result<Self> {
        let listener = UdpSocket::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let shutdown = listener.try_clone()?;
        let token = crate::native_guard::session_id().map_err(std::io::Error::other)?;
        let stopped = Arc::new(AtomicBool::new(false));
        let receiver_stop = Arc::clone(&stopped);
        let receiver_token = token.clone();
        let watch = Arc::new(Mutex::new(None));
        let receiver_watch = Arc::clone(&watch);
        let receiver_publisher = publisher;
        thread::Builder::new()
            .name("native-ui-events".into())
            .spawn(move || {
                let mut packet = [0; 128];
                let mut delay = MIN_SCAN_DELAY;
                while !receiver_stop.load(Ordering::Acquire) {
                    if listener.set_read_timeout(Some(delay)).is_err() {
                        break;
                    }
                    match listener.recv_from(&mut packet) {
                        Ok((0, _)) if receiver_stop.load(Ordering::Acquire) => break,
                        Ok((length, _)) => {
                            let Ok(message) = std::str::from_utf8(&packet[..length]) else {
                                continue;
                            };
                            if message == "wake" {
                                delay = MIN_SCAN_DELAY;
                                continue;
                            }
                            if let Some((received_token, surface)) = message.split_once(':')
                                && received_token == receiver_token
                                && let Some(surface) = UiSurface::from_wire(surface)
                            {
                                receiver_publisher.publish(UiEvent::invalidated(surface));
                                delay = MIN_SCAN_DELAY;
                            }
                        }
                        Err(error)
                            if matches!(
                                error.kind(),
                                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                            ) =>
                        {
                            if let Some(surface) = changed_surface(&receiver_watch) {
                                receiver_publisher.publish(UiEvent::invalidated(surface));
                                delay = MIN_SCAN_DELAY;
                            } else {
                                delay = next_scan_delay(delay, false);
                            }
                        }
                        Err(_) => break,
                    }
                }
            })?;
        Ok(Self {
            address,
            token,
            stopped,
            shutdown,
            watch,
        })
    }

    pub(super) fn watch_run_dir(&self, dir: &Path, surface: UiSurface) {
        if let Ok(mut watch) = self.watch.lock() {
            *watch = Some(FileWatch::new(dir, surface));
            let _ = self.shutdown.send_to(b"wake", self.address);
        }
    }

    pub(super) fn apply_to(&self, command: &mut Command, surface: UiSurface) {
        command
            .env("AC_UI_EVENT_ADDRESS", self.address.to_string())
            .env("AC_UI_EVENT_TOKEN", &self.token)
            .env("AC_UI_EVENT_SURFACE", surface.wire_name());
    }

    #[cfg(test)]
    pub(super) fn credentials(&self) -> (SocketAddr, String) {
        (self.address, self.token.clone())
    }
}

impl Drop for EventBridge {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        let _ = self.shutdown.send_to(&[], self.address);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileStamp {
    length: u64,
    modified: Option<SystemTime>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FileWatch {
    dir: PathBuf,
    surface: UiSurface,
    stamps: Vec<(PathBuf, Option<FileStamp>)>,
}

impl FileWatch {
    pub(super) fn new(dir: &Path, surface: UiSurface) -> Self {
        Self {
            dir: dir.to_path_buf(),
            surface,
            stamps: file_stamps(dir),
        }
    }

    pub(super) fn changed(&mut self) -> bool {
        let next = file_stamps(&self.dir);
        if next == self.stamps {
            return false;
        }
        self.stamps = next;
        true
    }
}

fn file_stamps(dir: &Path) -> Vec<(PathBuf, Option<FileStamp>)> {
    let mut paths = ["report.json", "state.sqlite", "state.sqlite-wal"]
        .into_iter()
        .map(|name| dir.join(name))
        .collect::<BTreeSet<_>>();
    collect_declared_worktree_paths(dir, &mut paths);
    paths
        .into_iter()
        .take(MAX_SCOPE_PATHS)
        .map(|path| {
            let stamp = fs::metadata(&path).ok().map(|metadata| FileStamp {
                length: metadata.len(),
                modified: metadata.modified().ok(),
            });
            (path, stamp)
        })
        .collect()
}

fn collect_declared_worktree_paths(dir: &Path, paths: &mut BTreeSet<PathBuf>) {
    let Ok(bytes) = fs::read(dir.join("report.json")) else {
        return;
    };
    let Ok(report) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return;
    };
    let worktrees = report["tasks"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|task| {
            Some((
                task["id"].as_str()?.to_owned(),
                PathBuf::from(task["cwd"].as_str()?),
            ))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    for task in report["plan"]["tasks"].as_array().into_iter().flatten() {
        let Some(cwd) = task["id"].as_str().and_then(|id| worktrees.get(id)) else {
            continue;
        };
        for path in task["paths"].as_array().into_iter().flatten() {
            let Some(path) = path.as_str().and_then(safe_scope_path) else {
                continue;
            };
            let target = cwd.join(path);
            collect_scope_tree(&target, 0, paths);
            if paths.len() >= MAX_SCOPE_PATHS {
                return;
            }
        }
    }
}

fn safe_scope_path(value: &str) -> Option<PathBuf> {
    let prefix = value.split(['*', '?', '[']).next()?.trim_end_matches('/');
    let path = if prefix.is_empty() {
        Path::new(".")
    } else {
        Path::new(prefix)
    };
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        })
    {
        return None;
    }
    Some(path.to_path_buf())
}

fn collect_scope_tree(path: &Path, depth: usize, paths: &mut BTreeSet<PathBuf>) {
    if paths.len() >= MAX_SCOPE_PATHS
        || !paths.insert(path.to_path_buf())
        || depth >= MAX_SCOPE_DEPTH
    {
        return;
    }
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    if !metadata.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries
        .flatten()
        .take(MAX_SCOPE_PATHS.saturating_sub(paths.len()))
    {
        collect_scope_tree(&entry.path(), depth + 1, paths);
    }
}

pub(super) fn changed_surface(watch: &Mutex<Option<FileWatch>>) -> Option<UiSurface> {
    watch.lock().ok().and_then(|mut watch| {
        let watch = watch.as_mut()?;
        watch.changed().then_some(watch.surface)
    })
}

pub(super) fn next_scan_delay(current: Duration, changed: bool) -> Duration {
    if changed {
        return MIN_SCAN_DELAY;
    }
    current.saturating_mul(2).min(MAX_SCAN_DELAY)
}

#[cfg(test)]
#[path = "../../../tests/support/ui_event_bridge.rs"]
mod tests;
