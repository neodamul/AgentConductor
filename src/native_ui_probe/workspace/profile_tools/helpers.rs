pub fn switch_profile(
    current: &Value,
    template: &Value,
    profiles: &[Value],
) -> Option<(Value, std::collections::BTreeMap<String, String>)> {
    let old_id = current["id"].as_str()?;
    let base = template["id"].as_str()?;
    let mut id = base.to_owned();
    let mut suffix = 2;
    while profiles
        .iter()
        .any(|profile| profile["id"] == id && profile["id"] != old_id)
    {
        id = format!("{base}-{suffix}");
        suffix += 1;
    }
    let mut switched = template.clone();
    switched["id"] = id.clone().into();
    let renames = if old_id != id {
        [(old_id.to_owned(), id)].into_iter().collect()
    } else {
        Default::default()
    };
    Some((switched, renames))
}

pub fn rename_references(
    profiles: &mut [Value],
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    renames: &std::collections::BTreeMap<String, String>,
) {
    let rename = |id: &mut String| {
        if let Some(next) = renames.get(id) {
            *id = next.clone();
        }
    };
    rename(coordinator);
    for entries in roles.values_mut() {
        for entry in entries {
            rename(&mut entry.agent);
        }
    }
    for profile in profiles {
        if let Some(id) = profile["fallback"]["agent"].as_str().map(str::to_owned) {
            let mut id = id;
            rename(&mut id);
            profile["fallback"]["agent"] = id.into();
        }
    }
}

pub fn metadata_request(event: &Event, revision: &str) -> Vec<u8> {
    let (kind, agent) = match event {
        Event::Discover(agent) => ("discover", agent),
        Event::Quota(agent) => ("quota", agent),
        Event::SaveProfiles
        | Event::BrowseExecutable
        | Event::ResolveExecutable(_)
        | Event::SetCatalog { .. } => {
            unreachable!()
        }
    };
    serde_json::to_vec(&serde_json::json!({"kind":kind,"agent":agent,"revision":revision}))
        .expect("metadata request is serializable")
}

pub fn profile_save_request(
    snapshot: &Value,
    profiles: &[Value],
    renames: &std::collections::BTreeMap<String, String>,
) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "kind":"save_profiles",
        "revision":snapshot["revision"],
        "profiles":profiles,
        "renames":renames
    }))
    .expect("profile save request is serializable")
}

pub fn remove_profile(
    profiles: &mut Vec<Value>,
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    editor: &mut Editor,
    id: &str,
) -> bool {
    let length = profiles.len();
    profiles.retain(|profile| profile["id"] != id);
    if profiles.len() == length {
        editor.remove_confirmation = None;
        return false;
    }
    if editor.id.as_deref() == Some(id) {
        editor.cancel();
    }
    editor.remove_confirmation = None;
    remove_references(profiles, coordinator, roles, id);
    true
}

pub fn remove_references(
    profiles: &mut [Value],
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    removed: &str,
) {
    if coordinator == removed {
        *coordinator = profiles
            .first()
            .and_then(|profile| profile["id"].as_str())
            .unwrap_or_default()
            .to_owned();
    }
    for profile in profiles {
        if profile["fallback"]["agent"] == removed {
            profile["fallback"] = Value::Null;
        }
    }
    roles.retain(|_, entries| {
        entries.retain(|entry| entry.agent != removed);
        !entries.is_empty()
    });
}

pub fn merge_catalogs(local: &mut [Value], saved: &[Value]) {
    for profile in local {
        let Some(observed) = saved.iter().find(|item| {
            item["id"] == profile["id"]
                && item["parser"] == profile["parser"]
                && item["executable"] == profile["executable"]
        }) else {
            continue;
        };
        if !observed["catalog"].is_null() {
            profile["catalog"] = observed["catalog"].clone();
        }
    }
}

use super::{Editor, Event};
use serde_json::Value;
