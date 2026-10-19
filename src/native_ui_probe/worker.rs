use eframe::egui;
use serde_json::Value;
use std::sync::{Arc, mpsc};

enum WorkerRequest {
    Dispatch(Vec<u8>),
    PickFolder,
    PickExecutable,
}

pub struct RequestWorker {
    requests: mpsc::SyncSender<WorkerRequest>,
    pub replies: mpsc::Receiver<Result<Value, String>>,
}

impl RequestWorker {
    pub fn new(room: Arc<crate::ui::Room>, context: egui::Context) -> Result<Self, String> {
        let (requests, queue) = mpsc::sync_channel::<WorkerRequest>(1);
        let (reply_tx, replies) = mpsc::channel();
        std::thread::Builder::new()
            .name("native-ui-requests".into())
            .spawn(move || {
                while let Ok(request) = queue.recv() {
                    let result = match request {
                        WorkerRequest::Dispatch(packet) => room.dispatch(&packet),
                        WorkerRequest::PickFolder => {
                            crate::ui::picker::pick_folder().and_then(|path| {
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
        Ok(Self { requests, replies })
    }

    pub fn send(&self, packet: &[u8]) -> Result<(), String> {
        self.requests
            .try_send(WorkerRequest::Dispatch(packet.to_vec()))
            .map_err(|error| error.to_string())
    }

    pub fn pick_folder(&self) -> Result<(), String> {
        self.requests
            .try_send(WorkerRequest::PickFolder)
            .map_err(|error| error.to_string())
    }

    pub fn pick_executable(&self) -> Result<(), String> {
        self.requests
            .try_send(WorkerRequest::PickExecutable)
            .map_err(|error| error.to_string())
    }
}

