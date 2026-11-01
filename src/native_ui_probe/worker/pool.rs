use eframe::egui;
use serde_json::Value;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};

const QUEUE_CAPACITY: usize = 16;

struct Tagged<T> {
    id: u64,
    request: T,
}

pub(super) struct Pool<T> {
    senders: Vec<mpsc::SyncSender<Tagged<T>>>,
    next_id: AtomicU64,
    latest_id: Arc<AtomicU64>,
}

pub struct Replies {
    receiver: mpsc::Receiver<(u64, Result<Value, String>)>,
    latest_id: Arc<AtomicU64>,
}

impl Replies {
    pub fn try_recv(&self) -> Result<Result<Value, String>, mpsc::TryRecvError> {
        loop {
            let (id, reply) = self.receiver.try_recv()?;
            if id == self.latest_id.load(Ordering::Acquire) {
                return Ok(reply);
            }
        }
    }

    pub fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Result<Value, String>, mpsc::RecvTimeoutError> {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let (id, reply) = self.receiver.recv_timeout(remaining)?;
            if id == self.latest_id.load(Ordering::Acquire) {
                return Ok(reply);
            }
        }
    }
}

impl<T: Send + 'static> Pool<T> {
    pub fn new(
        context: egui::Context,
        worker_counts: &[usize],
        dispatch: Arc<dyn Fn(T) -> Result<Value, String> + Send + Sync>,
    ) -> Result<(Self, Replies), String> {
        let (reply_tx, receiver) = mpsc::channel();
        let latest_id = Arc::new(AtomicU64::new(0));
        let mut senders = Vec::with_capacity(worker_counts.len());
        for (lane, worker_count) in worker_counts.iter().copied().enumerate() {
            let (sender, queue) = mpsc::sync_channel::<Tagged<T>>(QUEUE_CAPACITY);
            senders.push(sender);
            let queue = Arc::new(Mutex::new(queue));
            for worker in 0..worker_count {
                spawn_worker(
                    lane,
                    worker,
                    Arc::clone(&queue),
                    reply_tx.clone(),
                    context.clone(),
                    Arc::clone(&dispatch),
                )?;
            }
        }
        Ok((
            Self {
                senders,
                next_id: AtomicU64::new(0),
                latest_id: Arc::clone(&latest_id),
            },
            Replies {
                receiver,
                latest_id,
            },
        ))
    }

    pub fn send(&self, lane: usize, request: T) -> Result<(), String> {
        let sender = self
            .senders
            .get(lane)
            .ok_or_else(|| "foreground request lane is unavailable".to_owned())?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        sender
            .try_send(Tagged { id, request })
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => "foreground request lane is busy".to_owned(),
                mpsc::TrySendError::Disconnected(_) => {
                    "foreground request lane is unavailable".to_owned()
                }
            })?;
        self.latest_id.fetch_max(id, Ordering::Release);
        Ok(())
    }
}

fn spawn_worker<T: Send + 'static>(
    lane: usize,
    worker: usize,
    queue: Arc<Mutex<mpsc::Receiver<Tagged<T>>>>,
    replies: mpsc::Sender<(u64, Result<Value, String>)>,
    context: egui::Context,
    dispatch: Arc<dyn Fn(T) -> Result<Value, String> + Send + Sync>,
) -> Result<(), String> {
    std::thread::Builder::new()
        .name(format!("native-ui-request-{lane}-{worker}"))
        .spawn(move || {
            loop {
                let request = {
                    let receiver = queue
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    receiver.recv()
                };
                let Ok(request) = request else { break };
                let _ = replies.send((request.id, dispatch(request.request)));
                context.request_repaint();
            }
        })
        .map(|_| ())
        .map_err(|error| error.to_string())
}
