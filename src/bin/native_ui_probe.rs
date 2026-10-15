use agentconductor::ui::Room;
use std::{io, sync::Arc};

fn main() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (room, room_view) = match args.as_slice() {
        [] => {
            let root = agentconductor::desktop::workspace()?;
            (Arc::new(Room::startup(&root, &executable)?), false)
        }
        [flag, run] if flag == "--room" => (
            Arc::new(Room::open(std::path::Path::new(run), &executable)?),
            true,
        ),
        _ => return Err("usage: agentconductor-native-ui-probe [--room <run-dir>]".into()),
    };
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([720.0, 480.0])
            .with_title(if room_view {
                "AgentConductor Native Agent Room Probe"
            } else {
                "AgentConductor Native Workspace Probe"
            }),
        ..Default::default()
    };
    eframe::run_native(
        "AgentConductor Native UI Probe",
        options,
        Box::new(move |context| {
            let probe = if room_view {
                agentconductor::native_ui_probe::Probe::connect_room(room, context.egui_ctx.clone())
            } else {
                agentconductor::native_ui_probe::Probe::connect(room, context.egui_ctx.clone())
            };
            probe
                .map(|probe| Box::new(probe) as Box<dyn eframe::App>)
                .map_err(|error| Box::new(io::Error::other(error)) as _)
        }),
    )
    .map_err(|error| error.to_string())
}
