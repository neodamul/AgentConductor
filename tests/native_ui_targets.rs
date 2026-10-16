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
