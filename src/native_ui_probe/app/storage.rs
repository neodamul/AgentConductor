use std::{
    collections::hash_map::DefaultHasher,
    ffi::OsStr,
    hash::{Hash, Hasher},
};

pub fn app_storage_key(screenshot_capture: bool, screenshot_path: Option<&OsStr>) -> String {
    if !screenshot_capture {
        return "AgentConductor".into();
    }
    let Some(path) = screenshot_path else {
        return "AgentConductor screenshot capture".into();
    };
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("AgentConductor screenshot capture {:016x}", hasher.finish())
}
