use agentconductor::ui::Room;
#[path = "native_ui_probe/args.rs"]
mod args;

fn main() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (room, surface) = match args.as_slice() {
        [] => {
            let root = agentconductor::desktop::workspace()?;
            (
                std::sync::Arc::new(Room::startup(&root, &executable)?),
                agentconductor::native_ui_probe::Surface::Workspace,
            )
        }
        [flag, run] if flag == "--room" => (
            std::sync::Arc::new(Room::open(std::path::Path::new(run), &executable)?),
            agentconductor::native_ui_probe::Surface::AgentRoom,
        ),
        _ => {
            let workspace =
                args::workspace_path(&args)?.unwrap_or(agentconductor::desktop::workspace()?);
            (
                std::sync::Arc::new(Room::startup(&workspace, &executable)?),
                agentconductor::native_ui_probe::Surface::Workspace,
            )
        }
    };
    agentconductor::native_ui_probe::open_window(room, surface)
}
