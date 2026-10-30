use super::{EventPublisher, UiEvent, UiSurface, bridge};
use std::{
    path::Path,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

pub(super) struct RecoveryObserver {
    stopped: Arc<AtomicBool>,
    wake: Arc<Condvar>,
    watch: Arc<Mutex<Option<bridge::FileWatch>>>,
}

impl RecoveryObserver {
    pub(super) fn new(publisher: EventPublisher) -> std::io::Result<Self> {
        let stopped = Arc::new(AtomicBool::new(false));
        let wake = Arc::new(Condvar::new());
        let watch = Arc::new(Mutex::new(None));
        let thread_stop = Arc::clone(&stopped);
        let thread_wake = Arc::clone(&wake);
        let thread_watch = Arc::clone(&watch);
        let thread_publisher = publisher;
        thread::Builder::new()
            .name("native-ui-recovery".into())
            .spawn(move || {
                let mut delay = bridge::MIN_SCAN_DELAY;
                while !thread_stop.load(Ordering::Acquire) {
                    let Ok(guard) = thread_watch.lock() else {
                        break;
                    };
                    let Ok((guard, _)) = thread_wake.wait_timeout(guard, delay) else {
                        break;
                    };
                    drop(guard);
                    if thread_stop.load(Ordering::Acquire) {
                        break;
                    }
                    let changed = bridge::changed_surface(&thread_watch);
                    if let Some(surface) = changed {
                        thread_publisher.publish(UiEvent::invalidated(surface));
                    }
                    delay = bridge::next_scan_delay(delay, changed.is_some());
                }
            })?;
        Ok(Self {
            stopped,
            wake,
            watch,
        })
    }

    pub(super) fn watch_run_dir(&self, dir: &Path, surface: UiSurface) {
        if let Ok(mut watch) = self.watch.lock() {
            *watch = Some(bridge::FileWatch::new(dir, surface));
            self.wake.notify_one();
        }
    }
}

impl Drop for RecoveryObserver {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        self.wake.notify_one();
    }
}
