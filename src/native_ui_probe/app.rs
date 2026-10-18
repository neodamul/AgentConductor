use eframe::egui;
use std::sync::Arc;

use super::{Probe, Surface, room, workspace};

fn renderer() -> eframe::Renderer {
    #[cfg(windows)]
    {
        eframe::Renderer::Wgpu
    }
    #[cfg(not(windows))]
    {
        eframe::Renderer::Glow
    }
}

pub fn open_window(room: Arc<crate::ui::Room>, surface: Surface) -> Result<(), String> {
    let room_view = surface == Surface::AgentRoom;
    let options = eframe::NativeOptions {
        renderer: renderer(),
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1140.0, 740.0])
            .with_min_inner_size([840.0, 580.0])
            .with_icon(Arc::new(egui::IconData {
                rgba: include_bytes!("../ui/assets/ac-64.rgba").to_vec(),
                width: 64,
                height: 64,
            }))
            .with_title(if room_view {
                "AgentConductor — Agent Room"
            } else {
                "AgentConductor"
            }),
        ..Default::default()
    };
    eframe::run_native(
        "AgentConductor",
        options,
        Box::new(move |context| {
            Probe::connect_surface(room, context.egui_ctx.clone(), surface)
                .map(|probe| Box::new(probe) as Box<dyn eframe::App>)
                .map_err(|error| Box::new(std::io::Error::other(error)) as _)
        }),
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "../../tests/native_ui_probe/renderer.rs"]
mod tests;

impl Probe {
    pub fn update_ui(&mut self, ui: &mut egui::Ui) {
        self.poll();
        self.navigation.appearance.apply(ui.ctx());
        if self.room_view.as_mut().is_some_and(|room_view| {
            room::refresh_due(
                ui.ctx(),
                self.snapshot.as_ref(),
                self.pending.is_some(),
                &mut room_view.next_refresh,
            )
        }) {
            self.refresh_room();
        }
        if self.room_view.is_none() && self.starting_run.is_some() && self.pending.is_none() {
            let now = std::time::Instant::now();
            if let Some(deadline) = self.workspace_next_refresh.filter(|due| *due > now) {
                ui.ctx().request_repaint_after(deadline - now);
            } else {
                self.workspace_next_refresh = Some(now + std::time::Duration::from_millis(500));
                self.refresh_workspace();
            }
        }
        if self.room_view.is_none()
            && self.workspace_step == 2
            && self.starting_run.is_none()
            && self
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot["job"]["state"] == "running")
            && room::refresh_due(
                ui.ctx(),
                self.snapshot.as_ref(),
                self.pending.is_some(),
                &mut self.workspace_next_refresh,
            )
        {
            self.refresh_workspace();
        }
        if self.room_view.is_none()
            && self.workspace_step == 1
            && self
                .snapshot
                .as_ref()
                .is_some_and(workspace::profile_tools::metadata_running)
            && room::refresh_due(
                ui.ctx(),
                self.snapshot.as_ref(),
                self.pending.is_some(),
                &mut self.workspace_next_refresh,
            )
        {
            self.refresh_metadata();
        }
        if self.room_view.is_none()
            && (self.mcp_editor.discovering() || self.mcp_sources.previewing())
            && self.snapshot.as_ref().is_some_and(|snapshot| {
                snapshot["job"]["kind"] == "mcp" && snapshot["job"]["state"] == "running"
            })
            && room::refresh_due(
                ui.ctx(),
                self.snapshot.as_ref(),
                self.pending.is_some(),
                &mut self.workspace_next_refresh,
            )
        {
            self.refresh_mcp_snapshot();
        }
        self.show(ui);
    }
}

impl eframe::App for Probe {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.update_ui(ui);
    }
}
