use agentconductor::{desktop, ui::Room};
use std::{io, sync::Arc};

fn main() -> Result<(), String> {
    let root = desktop::workspace()?;
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let room = Arc::new(Room::startup(&root, &executable)?);
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([720.0, 480.0])
            .with_title("AgentConductor Native UI Probe"),
        ..Default::default()
    };
    eframe::run_native(
        "AgentConductor Native UI Probe",
        options,
        Box::new(move |context| {
            agentconductor::native_ui_probe::Probe::connect(room, context.egui_ctx.clone())
                .map(|probe| Box::new(probe) as Box<dyn eframe::App>)
                .map_err(|error| Box::new(io::Error::other(error)) as _)
        }),
    )
    .map_err(|error| error.to_string())
}
