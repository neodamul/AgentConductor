#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    List,
    Cache,
    Mcp,
    Messenger,
    WorkspaceSharing,
    UpdateRepository,
    Updates,
    BackToRun,
    DiscardChanges { return_to_run: bool },
    Create(String),
    ConfirmCreate(String),
    Open(String),
    ConfirmOpen(String),
    Forget(String),
    Delete { path: String, name: String },
}

pub fn request(event: &Event) -> Vec<u8> {
    let value = match event {
        Event::List => serde_json::json!({"kind":"saved_workspaces"}),
        Event::Cache => serde_json::json!({"kind":"cache_preview"}),
        Event::Mcp
        | Event::Messenger
        | Event::WorkspaceSharing
        | Event::UpdateRepository
        | Event::Updates => {
            unreachable!("settings use their own requests")
        }
        Event::BackToRun | Event::DiscardChanges { .. } => {
            unreachable!("workspace header actions do not send navigation requests")
        }
        Event::Create(name) | Event::ConfirmCreate(name) => {
            serde_json::json!({"kind":"create_workspace","name":name})
        }
        Event::Open(path) | Event::ConfirmOpen(path) => {
            serde_json::json!({"kind":"open_workspace","path":path})
        }
        Event::Forget(path) => serde_json::json!({"kind":"forget_workspace","path":path}),
        Event::Delete { path, name } => serde_json::json!({
            "kind":"delete_workspace", "path":path, "name":name, "replacement":null
        }),
    };
    serde_json::to_vec(&value).expect("workspace navigation request is serializable")
}

pub fn needs_discard_confirmation(event: &Event, dirty: bool) -> bool {
    dirty && matches!(event, Event::Create(_) | Event::Open(_))
}
