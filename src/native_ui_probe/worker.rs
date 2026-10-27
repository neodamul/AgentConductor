use eframe::egui;
use serde_json::Value;
use std::sync::{Arc, mpsc};

enum WorkerRequest {
    Dispatch(Vec<u8>),
    PickFolder(crate::ui::picker::FolderPurpose),
    PickPlanFile,
    PickExecutable,
}

pub struct RequestWorker {
    requests: mpsc::SyncSender<WorkerRequest>,
    pub replies: mpsc::Receiver<Result<Value, String>>,
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
        let (requests, queue) = mpsc::sync_channel::<WorkerRequest>(1);
        let (reply_tx, replies) = mpsc::channel();
        std::thread::Builder::new()
            .name("native-ui-requests".into())
            .spawn(move || {
                while let Ok(request) = queue.recv() {
                    let result = match request {
                        WorkerRequest::Dispatch(packet) => room.dispatch(&packet),
                        WorkerRequest::PickFolder(purpose) => {
                            crate::ui::picker::pick_folder(purpose).and_then(|path| {
                                let path = path
                                    .map(|path| {
                                        path.into_os_string().into_string().map_err(|_| {
                                            crate::ui::messages::text(
                                                "native.non_utf_8_folder_path",
                                            )
                                            .to_owned()
                                        })
                                    })
                                    .transpose()?;
                                Ok(serde_json::json!({"picked_folder": path}))
                            })
                        }
                        WorkerRequest::PickPlanFile => {
                            crate::ui::picker::pick_file().and_then(|path| {
                                let path = path
                                    .map(|path| {
                                        path.into_os_string().into_string().map_err(|_| {
                                            crate::ui::messages::text(
                                                "native.non_utf_8_folder_path",
                                            )
                                            .to_owned()
                                        })
                                    })
                                    .transpose()?;
                                Ok(serde_json::json!({"picked_plan_file": path}))
                            })
                        }
                        WorkerRequest::PickExecutable => crate::ui::picker::pick_executable()
                            .and_then(|path| {
                                let path = path
                                    .map(|path| {
                                        path.into_os_string()
                                            .into_string()
                                            .map_err(|_| "Executable path is not UTF-8".to_owned())
                                    })
                                    .transpose()?;
                                Ok(serde_json::json!({"picked_executable":path}))
                            }),
                    };
                    let _ = reply_tx.send(result);
                    context.request_repaint();
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(Self {
            requests,
            replies,
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
