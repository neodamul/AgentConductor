#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::{profile_tools, sidebar};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::{Value, json};

#[test]
fn agent_cards_expose_localized_move_controls_with_disabled_boundaries() {
    for language in ["en", "ko"] {
        let profiles = vec![
            json!({"id":"claude","parser":"claude"}),
            json!({"id":"codex","parser":"codex"}),
            json!({"id":"company","parser":"custom"}),
        ];
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(210.0, 800.0),
                )),
                ..Default::default()
            },
            |ui| {
                sidebar::show(&profiles, &json!({}), language, true, false, ui);
            },
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        for (id, up_disabled, down_disabled) in [
            ("claude", true, false),
            ("codex", false, false),
            ("company", false, true),
        ] {
            for (key, disabled) in [
                ("startup.move_up", up_disabled),
                ("startup.move_down", down_disabled),
            ] {
                let name = format!(
                    "{id}: {}",
                    agentconductor::ui::messages::in_language(language, key)
                );
                let node = nodes
                    .iter()
                    .map(|(_, node)| node)
                    .find(|node| node.role() == Role::Button && node.label() == Some(name.as_str()))
                    .unwrap_or_else(|| panic!("missing reorder action {name}"));
                assert_eq!(node.is_disabled(), disabled, "{name}");
                let bounds = node.bounds().unwrap();
                assert!(bounds.x0 >= 0.0 && bounds.x1 <= 210.0, "{name}: {bounds:?}");
            }
        }
    }
}

#[test]
fn move_buttons_and_drag_emit_the_same_order_change() {
    let profiles = vec![
        json!({"id":"claude","parser":"claude"}),
        json!({"id":"codex","parser":"codex"}),
        json!({"id":"company","parser":"custom"}),
    ];
    for drag in [false, true] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let (_, mut output) = frame(&context, &profiles, true, false, Vec::new());
        clear(&mut output);
        let (_, mut output) = frame(&context, &profiles, true, false, Vec::new());
        let start = position(
            &output,
            if drag {
                "company: Drag to reorder"
            } else {
                "claude: Move down"
            },
        );
        let target = position(&output, "claude");
        clear(&mut output);
        let (_, mut output) = frame(
            &context,
            &profiles,
            true,
            false,
            vec![Event::PointerMoved(start), pointer(start, true)],
        );
        clear(&mut output);
        if drag {
            for point in [start + egui::vec2(12.0, 0.0), target] {
                let (_, mut output) = frame(
                    &context,
                    &profiles,
                    true,
                    false,
                    vec![Event::PointerMoved(point)],
                );
                clear(&mut output);
            }
        }
        let release = if drag { target } else { start };
        let (event, mut output) = frame(
            &context,
            &profiles,
            true,
            false,
            vec![pointer(release, false)],
        );
        clear(&mut output);
        let (agent, index) = if drag { ("company", 0) } else { ("claude", 1) };
        assert_eq!(event, Some(sidebar::Event::MoveAgent(agent.into(), index)));
        let mut reordered = profiles.clone();
        assert!(profile_tools::reorder_profiles(
            &mut reordered,
            agent,
            index
        ));
        assert_eq!(reordered[index]["id"], agent);
    }
}

#[test]
fn profile_edits_and_pending_work_disable_all_order_controls() {
    let profiles = vec![json!({"id":"first"}), json!({"id":"second"})];
    for (editable, editing) in [(false, false), (true, true)] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let (_, mut output) = frame(&context, &profiles, editable, editing, Vec::new());
        clear(&mut output);
        let actions = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .map(|(_, node)| node)
            .filter(|node| node.label().is_some_and(|label| label.contains(": Move ")))
            .collect::<Vec<_>>();
        assert_eq!(actions.len(), 4);
        assert!(actions.iter().all(|node| node.is_disabled()));
    }
}

#[test]
fn reordered_profiles_survive_reopen_without_changing_references_or_secrets() {
    use agentconductor::ui::setup::Startup;
    let root = std::env::temp_dir().join(format!(
        "ac-agent-order-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&root, exe).unwrap();
    let snapshot = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut profiles = ["first", "second", "third"]
        .map(|id| {
            json!({
                "id":id,"parser":"custom","executable":exe,"args":["--help"],
            "permission_mode":"external_sandbox","quota_group":id,"environment":{"TEST_PRIVATE_VALUE":"retained"}
            })
        })
        .to_vec();
    profiles[0]["fallback"] = json!({"agent":"third"});
    startup
        .dispatch(
            json!({"kind":"save","revision":snapshot["revision"],"config":{
                "profiles":profiles,"repository":"","goal":"Retain goal","coordinator":"second",
                "workspace_mode":"shared","roles":{"testing":["third","first"]}
            }})
            .to_string()
            .as_bytes(),
        )
        .unwrap();
    let before = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut ordered = before["config"]["profiles"].as_array().unwrap().clone();
    assert!(!profile_tools::reorder_profiles(&mut ordered, "missing", 0));
    assert!(!profile_tools::reorder_profiles(&mut ordered, "third", 3));
    assert!(!profile_tools::reorder_profiles(&mut ordered, "third", 2));
    assert!(profile_tools::reorder_profiles(&mut ordered, "third", 0));
    let request = profile_tools::profile_save_request(&before, &ordered, &Default::default());
    assert_eq!(startup.dispatch(&request).unwrap()["saved"], true);
    assert!(
        startup.dispatch(&request).is_err(),
        "stale reorder must be rejected"
    );
    drop(startup);
    let reopened = Startup::open(&root, exe).unwrap();
    let config = reopened.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["config"].clone();
    let ids = config["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["third", "first", "second"]);
    assert_eq!(config["coordinator"], "second");
    assert_eq!(config["roles"], before["config"]["roles"]);
    assert_eq!(config["profiles"][1]["fallback"]["agent"], "third");
    let saved: Value =
        serde_json::from_slice(&std::fs::read(root.join("startup.json")).unwrap()).unwrap();
    assert!(
        saved["config"]["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["environment"]["TEST_PRIVATE_VALUE"] == "retained")
    );
    drop(reopened);
    std::fs::remove_dir_all(root).unwrap();
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn position(output: &egui::FullOutput, label: &str) -> Pos2 {
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing {label}"));
    Pos2::new(
        (bounds.x0 + bounds.x1) as f32 / 2.0,
        (bounds.y0 + bounds.y1) as f32 / 2.0,
    )
}

fn frame(
    context: &egui::Context,
    profiles: &[Value],
    editable: bool,
    editing: bool,
    events: Vec<Event>,
) -> (Option<sidebar::Event>, egui::FullOutput) {
    let mut action = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(900.0, 800.0),
            )),
            events,
            ..Default::default()
        },
        |ui| action = sidebar::show(profiles, &json!({}), "en", editable, editing, ui),
    );
    (action, output)
}
