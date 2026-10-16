use agentconductor::ui::Room;
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
        _ => return Err("usage: agentconductor-native-ui-probe [--room <run-dir>]".into()),
    };
    agentconductor::native_ui_probe::open_window(room, surface)
}
