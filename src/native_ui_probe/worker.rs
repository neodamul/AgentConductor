use eframe::egui;
use serde_json::Value;
use std::sync::{Arc, mpsc};

type Handler = Arc<dyn Fn(WorkerRequest) -> Result<Value, String> + Send + Sync>;
#[cfg(test)]
type TestDispatch = Arc<dyn Fn(&[u8]) -> Result<Value, String> + Send + Sync>;

enum WorkerRequest {
    Dispatch(Vec<u8>),
    PickFolder(crate::ui::picker::FolderPurpose),
    PickPlanFile,
    PickExecutable,
}

pub(super) fn initial_snapshot(room_view: bool) -> &'static [u8] {
    if room_view {
        br#"{"kind":"snapshot","icons":true}"#
    } else {
        br#"{"kind":"snapshot","icons":true,"chat_history":true,"chat_result":true,"optimize_result":true}"#
    }
}

pub struct RequestWorker {
    requests: mpsc::SyncSender<WorkerRequest>,
    background_requests: mpsc::SyncSender<WorkerRequest>,
    pub replies: mpsc::Receiver<Result<Value, String>>,
    pub background_replies: mpsc::Receiver<Result<Value, String>>,
    pub events: mpsc::Receiver<crate::ui::events::Event>,
    room_view: bool,
}

impl RequestWorker {
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
        let wake_context = context.clone();
        let subscription = room.subscribe_events(move || wake_context.request_repaint())?;
        let source = subscription.id;
        let handler: Handler = Arc::new(move |request| handle(&room, source, request));
        Self::with_handler(context, room_view, handler, subscription.events)
    }

    fn with_handler(
        context: egui::Context,
        room_view: bool,
        handler: Handler,
        events: mpsc::Receiver<crate::ui::events::Event>,
    ) -> Result<Self, String> {
        let (requests, queue) = mpsc::sync_channel::<WorkerRequest>(1);
        let (background_requests, background_queue) = mpsc::sync_channel::<WorkerRequest>(1);
        let (reply_tx, replies) = mpsc::channel();
        let (background_reply_tx, background_replies) = mpsc::channel();
        spawn_lane(
            "native-ui-requests",
            queue,
            reply_tx,
            context.clone(),
            handler.clone(),
        )?;
        spawn_lane(
            "native-ui-background-refresh",
            background_queue,
            background_reply_tx,
            context,
            handler,
        )?;
        Ok(Self {
            requests,
            background_requests,
            replies,
            background_replies,
            events,
            room_view,
        })
    }

    #[cfg(test)]
    pub(super) fn with_dispatch_for_test(
        context: egui::Context,
        dispatch: TestDispatch,
    ) -> Result<Self, String> {
        let handler: Handler = Arc::new(move |request| match request {
            WorkerRequest::Dispatch(packet) => dispatch(&packet),
            _ => unreachable!("test worker only dispatches JSON requests"),
        });
        let (_, events) = mpsc::channel();
        Self::with_handler(context, false, handler, events)
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

    pub fn send(&self, packet: &[u8]) -> Result<(), String> {
        let packet = if self.room_view {
            Self::route_room_request(packet)?
        } else {
            packet.to_vec()
        };
        self.requests
            .try_send(WorkerRequest::Dispatch(packet))
            .map_err(|error| error.to_string())
    }

    pub fn send_background(&self, packet: &[u8]) -> Result<(), String> {
        let packet = if self.room_view {
            Self::route_room_request(packet)?
        } else {
            packet.to_vec()
        };
        self.background_requests
            .try_send(WorkerRequest::Dispatch(packet))
            .map_err(|error| error.to_string())
    }

    pub fn send_initial_snapshot(&self) -> Result<(), String> {
        self.send(initial_snapshot(self.room_view))
    }

    pub fn pick_folder(&self, purpose: crate::ui::picker::FolderPurpose) -> Result<(), String> {
        self.requests
            .try_send(WorkerRequest::PickFolder(purpose))
            .map_err(|error| error.to_string())
    }

    pub fn pick_plan_file(&self) -> Result<(), String> {
        self.requests
            .try_send(WorkerRequest::PickPlanFile)
            .map_err(|error| error.to_string())
    }

    pub fn pick_executable(&self) -> Result<(), String> {
        self.requests
            .try_send(WorkerRequest::PickExecutable)
            .map_err(|error| error.to_string())
    }
}

fn spawn_lane(
    name: &str,
    queue: mpsc::Receiver<WorkerRequest>,
    replies: mpsc::Sender<Result<Value, String>>,
    context: egui::Context,
    handler: Handler,
) -> Result<(), String> {
    std::thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            while let Ok(request) = queue.recv() {
                let _ = replies.send(handler(request));
                context.request_repaint();
            }
        })
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn handle(room: &crate::ui::Room, source: u64, request: WorkerRequest) -> Result<Value, String> {
    match request {
        WorkerRequest::Dispatch(packet) => room.dispatch_from(Some(source), &packet),
        WorkerRequest::PickFolder(purpose) => {
            crate::ui::picker::pick_folder(purpose).and_then(|path| {
                let path = path
                    .map(|path| os_string(path.into_os_string(), "native.non_utf_8_folder_path"))
                    .transpose()?;
                Ok(serde_json::json!({"picked_folder": path}))
            })
        }
        WorkerRequest::PickPlanFile => crate::ui::picker::pick_file().and_then(|path| {
            let path = path
                .map(|path| os_string(path.into_os_string(), "native.non_utf_8_folder_path"))
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
            Ok(serde_json::json!({"picked_executable": path}))
        }),
    }
}

fn os_string(value: std::ffi::OsString, message: &'static str) -> Result<String, String> {
    value
        .into_string()
        .map_err(|_| crate::ui::messages::text(message).to_owned())
}

#[cfg(test)]
#[path = "../../tests/native_ui_probe/worker_concurrency.rs"]
mod tests;
