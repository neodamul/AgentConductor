use super::*;
use eframe::egui::{Event, Modifiers, PointerButton, Pos2, accesskit::Role};

fn frame(probe: &mut Probe, ctx: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1140.0, 900.0),
            )),
            events,
            ..Default::default()
        },
        |ui| super::super::show(probe, ui),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output
}

fn click(probe: &mut Probe, ctx: &egui::Context, target: &str) {
    frame(probe, ctx, vec![]);
    let output = frame(probe, ctx, vec![]);
    let bounds = output
        .platform_output
        .accesskit_update
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(target))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing button {target}"));
    let pos = Pos2::new(
        (bounds.x0 + bounds.x1) as f32 / 2.0,
        (bounds.y0 + bounds.y1) as f32 / 2.0,
    );
    for pressed in [true, false] {
        frame(
            probe,
            ctx,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
}

#[test]
fn sidebar_add_opens_an_editor_without_inserting_a_card() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let mut probe = Probe {
        snapshot: Some(serde_json::json!({"templates":[{
            "id":"claude", "parser":"claude", "executable":"claude", "args":["-p"]
        }]})),
        ..Default::default()
    };
    let add = probe
        .navigation
        .appearance
        .text("startup.register_cli_agent");
    click(&mut probe, &ctx, add);
    assert!(
        probe.profile_editor.is_editing(),
        "Add must open the profile editor immediately"
    );
    assert!(
        probe.profiles.is_empty(),
        "an unconfirmed profile must not appear in the registry"
    );
    assert_eq!(
        probe.profile_editor.executable(),
        "aipro",
        "registration must retain the shipped custom-CLI default"
    );
}

struct Fixture {
    probe: Probe,
    room: Option<std::sync::Arc<crate::ui::Room>>,
    ctx: egui::Context,
    root: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ac-registration-{}",
            crate::native_guard::session_id().unwrap()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let room = std::sync::Arc::new(
            crate::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap(),
        );
        let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        let mut config = before["config"].clone();
        config["profiles"] = serde_json::json!([{
            "id":"company", "parser":"custom", "executable":"aipro", "args":[],
            "input_mode":"stdin", "permission_mode":"read_only", "max_sessions":1,
            "quota_refresh_minutes":5
        }]);
        config["coordinator"] = "company".into();
        config["goal"] = "Saved goal".into();
        config["name"] = "Saved name".into();
        config["repository"] = "".into();
        config["roles"] = serde_json::json!({"development":[{"agent":"company"}]});
        room.dispatch(
            serde_json::json!({"kind":"save", "revision":before["revision"], "config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut probe = Probe::connect(room.clone(), ctx.clone()).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while probe.snapshot.is_none() {
            probe.poll();
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        probe.snapshot.as_mut().unwrap()["templates"] =
            serde_json::json!([probe.profiles[0].clone()]);
        probe.goal = "Unsaved goal".into();
        probe.name = "Unsaved name".into();
        Self {
            probe,
            room: Some(room),
            ctx,
            root,
        }
    }

    fn click(&mut self, key: &'static str) {
        let target = self.probe.navigation.appearance.text(key);
        click(&mut self.probe, &self.ctx, target);
    }

    fn settle(&mut self) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while self.probe.pending.is_some() {
            self.probe.poll();
            assert!(
                std::time::Instant::now() < deadline,
                "profile worker timed out"
            );
            std::thread::yield_now();
        }
    }

    fn assert_draft(&self) {
        assert_eq!(self.probe.goal, "Unsaved goal");
        assert_eq!(self.probe.name, "Unsaved name");
        let stored = self
            .room
            .as_ref()
            .unwrap()
            .dispatch(br#"{"kind":"snapshot"}"#)
            .unwrap();
        assert_eq!(stored["config"]["goal"], "Saved goal");
        assert_eq!(stored["config"]["name"], "Saved name");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.probe.worker.take();
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn registration_save_persists_alone_and_survives_reopen() {
    let mut f = Fixture::new();
    f.click("startup.register_cli_agent");
    f.settle();
    assert_eq!(f.probe.profiles.len(), 1);
    f.probe.profile_editor.set_agent_id("helper");
    f.click("startup.save_profile");
    assert!(f.probe.profile_editor.is_editing());
    assert_eq!(f.probe.profiles.len(), 1);
    f.settle();
    assert!(!f.probe.profile_editor.is_editing());
    assert!(f.probe.profile_saved, "successful save must show a notice");
    assert_eq!(f.probe.profiles[1]["id"], "helper");
    assert_eq!(f.probe.coordinator, "company");
    assert_eq!(f.probe.roles["development"][0].agent, "company");
    f.assert_draft();
    f.probe.worker.take();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::sync::Arc::strong_count(f.room.as_ref().unwrap()) > 1 {
        assert!(
            std::time::Instant::now() < deadline,
            "old request worker did not exit"
        );
        std::thread::yield_now();
    }
    f.room.take();
    let reopened = crate::ui::Room::startup(&f.root, &std::env::current_exe().unwrap()).unwrap();
    assert_eq!(
        reopened.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["config"]["profiles"][1]["id"],
        "helper"
    );
}

#[test]
fn cancel_registration_does_not_leave_a_card_or_save_the_goal() {
    let mut f = Fixture::new();
    f.click("startup.register_cli_agent");
    f.settle();
    f.probe.profile_editor.set_agent_id("cancelled");
    f.click("startup.cancel");
    assert!(!f.probe.profile_editor.is_editing());
    assert_eq!(f.probe.profiles.len(), 1);
    assert_eq!(
        f.room
            .as_ref()
            .unwrap()
            .dispatch(br#"{"kind":"snapshot"}"#)
            .unwrap()["config"]["profiles"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    f.assert_draft();
}

#[test]
fn stale_save_retains_editor_and_refreshes_revision_for_retry() {
    let mut f = Fixture::new();
    f.click("startup.register_cli_agent");
    f.settle();
    f.probe.profile_editor.set_agent_id("helper");
    let before = f
        .room
        .as_ref()
        .unwrap()
        .dispatch(br#"{"kind":"snapshot"}"#)
        .unwrap();
    let mut profiles = before["config"]["profiles"].clone();
    profiles[0]["quota_refresh_minutes"] = 12.into();
    f.room.as_ref().unwrap().dispatch(serde_json::json!({"kind":"save_profiles", "revision":before["revision"], "profiles":profiles}).to_string().as_bytes()).unwrap();
    f.click("startup.save_profile");
    f.settle();
    assert!(f.probe.profile_editor.is_editing());
    assert_eq!(f.probe.profiles.len(), 1);
    assert_eq!(f.probe.profiles[0]["quota_refresh_minutes"], 12);
    let output = frame(&mut f.probe, &f.ctx, vec![]);
    let labels = output.platform_output.accesskit_update.unwrap().nodes;
    let prefix = f
        .probe
        .navigation
        .appearance
        .text("startup.profile_save_failed")
        .split('{')
        .next()
        .unwrap();
    assert!(
        labels
            .iter()
            .filter_map(|(_, n)| n.label())
            .any(|label| label.starts_with(prefix)),
        "save error must appear inside the editor"
    );
    assert!(!f.probe.profile_saved);
    f.assert_draft();
    f.click("startup.save_profile");
    f.settle();
    assert!(!f.probe.profile_editor.is_editing());
    assert_eq!(f.probe.profiles.len(), 2);
    assert_eq!(f.probe.profiles[0]["quota_refresh_minutes"], 12);
    f.assert_draft();
}

#[test]
fn edited_coordinator_is_renamed_only_after_confirmed_persistence() {
    let mut f = Fixture::new();
    handle_event(&mut f.probe, sidebar::Event::EditAgent("company".into()));
    f.probe.profile_editor.set_agent_id("renamed");
    f.click("startup.save_profile");
    assert_eq!(f.probe.coordinator, "company");
    assert_eq!(f.probe.profiles[0]["id"], "company");
    f.settle();
    assert_eq!(f.probe.coordinator, "renamed");
    assert_eq!(f.probe.roles["development"][0].agent, "renamed");
    assert_eq!(f.probe.profiles[0]["id"], "renamed");
    f.assert_draft();
}
