use eframe::egui;
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};

pub(in crate::native_ui_probe) type Dispatch = dyn Fn(&[u8]) -> Result<Value, String> + Send + Sync;

enum WorkerRequest {
    Dispatch(Vec<u8>),
    PickFolder(crate::ui::picker::FolderPurpose),
    PickPlanFile,
    PickExecutable,
}
struct TaggedRequest {
    id: u64,
    request: WorkerRequest,
}
const REQUEST_WORKERS: usize = 2;

fn run_request(request: WorkerRequest, dispatch: &Dispatch) -> Result<Value, String> {
    match request {
        WorkerRequest::Dispatch(packet) => dispatch(&packet),
        WorkerRequest::PickFolder(purpose) => {
            crate::ui::picker::pick_folder(purpose).and_then(|path| {
                let path = path
                    .map(|path| {
                        path.into_os_string().into_string().map_err(|_| {
                            crate::ui::messages::text("native.non_utf_8_folder_path").to_owned()
                        })
                    })
                    .transpose()?;
                Ok(serde_json::json!({"picked_folder": path}))
            })
        }
        WorkerRequest::PickPlanFile => crate::ui::picker::pick_file().and_then(|path| {
            let path = path
                .map(|path| {
                    path.into_os_string().into_string().map_err(|_| {
                        crate::ui::messages::text("native.non_utf_8_folder_path").to_owned()
                    })
                })
                .transpose()?;
            Ok(serde_json::json!({"picked_plan_file": path}))
        }),
        WorkerRequest::PickExecutable => crate::ui::picker::pick_executable().and_then(|path| {
            let path = path
                .map(|path| {
                    path.into_os_string()
                        .into_string()
                        .map_err(|_| "Executable path is not UTF-8".to_owned())
                })
                .transpose()?;
            Ok(serde_json::json!({"picked_executable":path}))
        }),
    }
}

pub(super) fn initial_snapshot(room_view: bool) -> &'static [u8] {
    if room_view {
        br#"{"kind":"snapshot","icons":true}"#
    } else {
        br#"{"kind":"snapshot","icons":true,"chat_history":true,"chat_result":true,"optimize_result":true}"#
    }
}

pub struct RequestWorker {
    requests: Vec<mpsc::SyncSender<TaggedRequest>>,
    pub replies: ReplyReceiver,
    latest_request: Arc<AtomicU64>,
    poll_requests: mpsc::SyncSender<Vec<u8>>,
    pub poll_replies: mpsc::Receiver<Result<Value, String>>,
    pub events: mpsc::Receiver<crate::ui::events::UiEvent>,
    room_view: bool,
}

pub struct ReplyReceiver {
    receiver: mpsc::Receiver<(u64, Result<Value, String>)>,
    latest_request: Arc<AtomicU64>,
}

impl ReplyReceiver {
    pub fn try_recv(&self) -> Result<Result<Value, String>, mpsc::TryRecvError> {
        loop {
            let (id, reply) = self.receiver.try_recv()?;
            if id == self.latest_request.load(Ordering::Acquire) {
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
            if id == self.latest_request.load(Ordering::Acquire) {
                return Ok(reply);
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/native_ui_probe/worker_concurrency.rs"]
mod concurrency_tests;
#[cfg(test)]
#[path = "../../tests/native_ui_probe/worker_events.rs"]
mod event_tests;
#[cfg(test)]
#[path = "../../tests/native_ui_probe/request_pool.rs"]
mod request_pool_tests;

impl RequestWorker {
    pub fn latest_request_id(&self) -> u64 {
        self.latest_request.load(Ordering::Acquire)
    }

    pub fn new(room: Arc<crate::ui::Room>, context: egui::Context) -> Result<Self, String> {
        Self::connect(room, context, false)
    }

    pub fn new_room(room: Arc<crate::ui::Room>, context: egui::Context) -> Result<Self, String> {
        Self::connect(room, context, true)
    }

    fn connect(
        room: Arc<crate::ui::Room>,
        context: egui::Context,
        room_view: bool,
    ) -> Result<Self, String> {
        let surface = if room_view {
            crate::ui::events::UiSurface::AgentRoom
        } else {
            crate::ui::events::UiSurface::Workspace
        };
        let events = room.subscribe_events(surface, context.clone());
        let dispatch_room = Arc::clone(&room);
        let dispatch: Arc<Dispatch> =
            Arc::new(move |packet| dispatch_room.dispatch_ui_request(packet));
        let mut worker = Self::connect_with(context, room_view, dispatch)?;
        worker.events = events;
        Ok(worker)
    }

    pub(in crate::native_ui_probe) fn connect_with(
        context: egui::Context,
        room_view: bool,
        dispatch: Arc<Dispatch>,
    ) -> Result<Self, String> {
        let (reply_tx, replies) = mpsc::channel();
        let latest_request = Arc::new(AtomicU64::new(0));
        let (poll_requests, poll_queue) = mpsc::sync_channel::<Vec<u8>>(1);
        let (poll_reply_tx, poll_replies) = mpsc::channel();
        let (_, events) = mpsc::sync_channel(1);
        let poll_dispatch = Arc::clone(&dispatch);
        let poll_context = context.clone();
        std::thread::Builder::new()
            .name("native-ui-polls".into())
            .spawn(move || {
                while let Ok(packet) = poll_queue.recv() {
                    let _ = poll_reply_tx.send(poll_dispatch(&packet));
                    poll_context.request_repaint();
                }
            })
            .map_err(|error| error.to_string())?;
        let mut requests = Vec::with_capacity(REQUEST_WORKERS);
        for index in 0..REQUEST_WORKERS {
            let (sender, queue) = mpsc::sync_channel::<TaggedRequest>(1);
            requests.push(sender);
            let request_dispatch = Arc::clone(&dispatch);
            let request_reply = reply_tx.clone();
            let request_context = context.clone();
            std::thread::Builder::new()
                .name(format!("native-ui-request-{index}"))
                .spawn(move || {
                    while let Ok(request) = queue.recv() {
                        let reply = run_request(request.request, request_dispatch.as_ref());
                        let _ = request_reply.send((request.id, reply));
                        request_context.request_repaint();
                    }
                })
                .map_err(|error| error.to_string())?;
        }
        Ok(Self {
            requests,
            replies: ReplyReceiver {
                receiver: replies,
                latest_request: Arc::clone(&latest_request),
            },
            latest_request,
            poll_requests,
            poll_replies,
            events,
            room_view,
        })
    }

    pub fn route_room_request(packet: &[u8]) -> Result<Vec<u8>, String> {
        let mut value: Value = serde_json::from_slice(packet).map_err(|error| error.to_string())?;
        let fields = value
            .as_object_mut()
            .ok_or("Room requests must be JSON objects")?;
        fields.insert("room".into(), Value::Bool(true));
        serde_json::to_vec(&value).map_err(|error| error.to_string())
    }

    pub fn set_room_view(&mut self, room_view: bool) {
        self.room_view = room_view;
    }

    fn send_request(&self, request: WorkerRequest) -> Result<(), String> {
        let lane = match &request {
            WorkerRequest::Dispatch(packet)
                if crate::ui::events::UiEvent::for_request(packet).is_none() =>
            {
                0
            }
            _ => 1,
        };
        let id = self.latest_request.load(Ordering::Relaxed).wrapping_add(1);
        self.requests[lane]
            .try_send(TaggedRequest { id, request })
            .map(|()| self.latest_request.store(id, Ordering::Release))
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => "foreground request lane is busy".into(),
                mpsc::TrySendError::Disconnected(_) => {
                    "foreground request lane is unavailable".into()
                }
            })
    }

    pub fn send(&self, packet: &[u8]) -> Result<(), String> {
        let packet = if self.room_view {
            Self::route_room_request(packet)?
        } else {
            packet.to_vec()
        };
        self.send_request(WorkerRequest::Dispatch(packet))
    }

    pub fn send_poll(&self, packet: &[u8]) -> Result<bool, String> {
        let packet = if self.room_view {
            Self::route_room_request(packet)?
        } else {
            packet.to_vec()
        };
        match self.poll_requests.try_send(packet) {
            Ok(()) => Ok(true),
            Err(mpsc::TrySendError::Full(_)) => Ok(false),
            Err(mpsc::TrySendError::Disconnected(_)) => {
                Err("background poll worker is unavailable".into())
            }
        }
    }

    pub fn send_initial_snapshot(&self) -> Result<(), String> {
        self.send(initial_snapshot(self.room_view))
    }

    pub fn pick_folder(&self, purpose: crate::ui::picker::FolderPurpose) -> Result<(), String> {
        self.send_request(WorkerRequest::PickFolder(purpose))
    }

    pub fn pick_plan_file(&self) -> Result<(), String> {
        self.send_request(WorkerRequest::PickPlanFile)
    }

    pub fn pick_executable(&self) -> Result<(), String> {
        self.send_request(WorkerRequest::PickExecutable)
    }
}
