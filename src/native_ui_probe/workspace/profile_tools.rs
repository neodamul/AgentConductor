use serde_json::Value;

mod catalog;
mod catalog_view;
mod editor;
mod helpers;
mod metadata;
mod view;

pub use catalog::{catalog_request, parse_manual_catalog};
pub use editor::Editor;
pub use helpers::{
    merge_catalogs, metadata_request, profile_save_request, remove_profile, remove_references,
    rename_references, switch_profile,
};
pub(super) use metadata::metadata_active;
pub use metadata::metadata_running;
pub use view::{RenderContext, show_in_language};

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Discover(String),
    Quota(String),
    SetCatalog {
        agent: String,
        catalog: crate::plan::ModelCatalog,
    },
    SaveProfiles,
    BrowseExecutable,
    ResolveExecutable(String),
}

pub(super) fn custom_template() -> Value {
    serde_json::json!({
        "id":"local-agent", "parser":"custom", "executable":"aipro",
        "args":[], "input_mode":"stdin", "permission_mode":"unverified",
        "max_sessions":3, "quota_refresh_minutes":5, "catalog":null
    })
}

pub fn reorder_profiles(profiles: &mut Vec<Value>, agent: &str, target: usize) -> bool {
    let Some(current) = profiles.iter().position(|profile| profile["id"] == agent) else {
        return false;
    };
    if target >= profiles.len() || current == target {
        return false;
    }
    let profile = profiles.remove(current);
    profiles.insert(target, profile);
    true
}

pub fn show(
    profiles: &mut Vec<Value>,
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    snapshot: &Value,
    editable: bool,
    editor: &mut Editor,
    ui: &mut eframe::egui::Ui,
) -> Option<Event> {
    view::show_in_language(
        profiles,
        coordinator,
        roles,
        editor,
        RenderContext {
            snapshot,
            editable,
            language: "en",
            show_registry: true,
        },
        ui,
    )
}
