use agentconductor::native_ui_probe::{
    Probe, RequestWorker, picked_folder, workspace_save_request,
};
use eframe::egui::{self, accesskit::Role};
use std::{sync::Arc, time::Duration};

#[path = "support/env.rs"]
mod test_env;

#[path = "native_ui_probe/part_1.rs"]
mod part_1;
#[path = "native_ui_probe/part_2.rs"]
mod part_2;
