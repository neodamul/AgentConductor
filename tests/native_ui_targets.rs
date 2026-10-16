#[test]
fn every_native_ui_integration_target_requires_the_optional_feature() {
    let manifest =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    for name in [
        "native_ui_probe",
        "native_ui_launch",
        "native_room_render",
        "native_ui_profiles",
        "native_ui_roles",
        "native_ui_tasks",
        "native_ui_pending",
        "native_ui_activity",
        "native_ui_tool_dialog",
        "native_ui_cache",
        "native_ui_workspaces",
        "native_ui_manual",
        "native_ui_updates",
    ] {
        let target = manifest
            .split("[[test]]")
            .skip(1)
            .find(|target| {
                target
                    .lines()
                    .any(|line| line.trim() == format!("name = \"{name}\""))
            })
            .unwrap_or_else(|| panic!("missing explicit test target: {name}"));
        assert!(
            target
                .lines()
                .any(|line| line.trim() == "required-features = [\"native-ui-probe\"]"),
            "{name} must require native-ui-probe"
        );
    }
}

#[test]
fn auto_discovered_native_ui_tests_are_feature_gated() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    for entry in std::fs::read_dir(root.join("tests")).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() || path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy();
        if name == "native_ui_targets.rs" {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        if !source.contains("agentconductor::native_ui_probe")
            && !source.lines().any(|line| line.starts_with("use eframe::"))
        {
            continue;
        }
        let gated = source
            .lines()
            .take(3)
            .any(|line| line.trim() == "#![cfg(feature = \"native-ui-probe\")]");
        let target_path = format!("path = \"tests/{name}\"");
        let declared = manifest.split("[[test]]").skip(1).any(|section| {
            section.lines().any(|line| line.trim() == target_path)
                && section
                    .lines()
                    .any(|line| line.trim() == "required-features = [\"native-ui-probe\"]")
        });
        assert!(
            gated || declared,
            "{} needs a native-ui-probe gate",
            path.display()
        );
    }
}
