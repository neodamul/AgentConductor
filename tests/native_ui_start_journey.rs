#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::{Probe, Surface};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, accesskit::Role};
use serde_json::json;
use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[path = "support/workflow.rs"]
mod support;
#[path = "native_ui_start_journey/ui_wait.rs"]
mod ui_wait;

#[test]
fn review_approval_shows_start_feedback_then_opens_the_started_run() {
    let (root, plan) = support::setup("shared");
    let mut agents = serde_json::to_value(&plan.agents).unwrap();
    agents[0]["args"] = json!(["task-slow"]);
    let mut planner = agents[0].clone();
    planner["id"] = json!("planner");
    planner["args"] = json!(["plan"]);
    planner["permission_mode"] = json!("read_only");
    agents.as_array_mut().unwrap().push(planner);
    let mock = PathBuf::from(plan.agents[0].executable.clone());
    let _cleanup = Cleanup::new(root.clone(), mock);
    let _home = IsolatedHome::new(root.join("home"));

    let room = Arc::new(
        agentconductor::ui::Room::startup(
            &root.join("desktop"),
            PathBuf::from(env!("CARGO_BIN_EXE_agentconductor")).as_path(),
        )
        .unwrap(),
    );
    let initial = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let save = json!({
        "kind":"save", "revision":initial["revision"], "config":{
            "name":"Native UI acceptance", "repository":root.join("repo"),
            "goal":"Implement and verify three files", "coordinator":"planner",
            "workspace_mode":"shared", "profiles":agents, "plan_rules":""
        }
    });
    room.dispatch(&serde_json::to_vec(&save).unwrap()).unwrap();
    let saved = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    room.dispatch(
        &serde_json::to_vec(&json!({"kind":"prepare","revision":saved["revision"]})).unwrap(),
    )
    .unwrap();
    wait_for(
        &room,
        |snapshot| snapshot["proposal"]["phase"] == "ready",
        "plan generation",
    );

    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe =
        Probe::connect_surface(room.clone(), context.clone(), Surface::Workspace).unwrap();
    let mut output = frame(&context, &mut probe, Vec::new());
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    ui_wait::wait_for_ui(&context, &mut probe, |labels| {
        labels.iter().any(|label| label == "1. Workspace settings")
    });

    let mut output = click(&context, &mut probe, "3. Review the plan");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    ui_wait::wait_for_ui(&context, &mut probe, |labels| {
        labels
            .iter()
            .any(|label| label == "Review exact plan before approving execution")
            && ["task-0 · one.txt", "task-1 · two.txt", "task-2 · three.txt"]
                .iter()
                .all(|task| labels.iter().any(|label| label.ends_with(task)))
    });

    output = click(
        &context,
        &mut probe,
        "Review exact plan before approving execution",
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = click(
        &context,
        &mut probe,
        "Confirm exact reviewed plan and start agent execution",
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut saw_pending = false;
    while Instant::now() < deadline {
        let text = visible_text(&output);
        if text.contains("Starting agent execution…") {
            saw_pending = true;
            break;
        }
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        thread::sleep(Duration::from_millis(25));
        output = frame(&context, &mut probe, Vec::new());
    }
    assert!(
        saw_pending,
        "start feedback did not appear: {}",
        visible_text(&output)
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = frame_with_height(&context, &mut probe, Vec::new(), 700.0);
    let pending_bounds = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Starting agent execution…" => {
                Some(Rect::from_min_size(text.pos, text.galley.size()))
            }
            _ => None,
        })
        .expect("pending status is rendered after resizing the window");
    assert!(
        pending_bounds.min.y >= 0.0 && pending_bounds.max.y <= 700.0,
        "pending status is clipped at short height: {pending_bounds:?}"
    );

    ui_wait::wait_for_ui_timeout(&context, &mut probe, Duration::from_secs(15), |labels| {
        labels
            .iter()
            .any(|label| label == "Load read-only team briefing")
    });
    drop(probe);
    drop(room);
    thread::sleep(Duration::from_millis(30));
}

struct Cleanup(PathBuf, PathBuf);

impl Cleanup {
    fn new(root: PathBuf, mock: PathBuf) -> Self {
        Self(root, mock)
    }
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
        let _ = fs::remove_file(&self.1);
    }
}

struct IsolatedHome(Option<OsString>);

impl IsolatedHome {
    fn new(path: PathBuf) -> Self {
        fs::create_dir_all(&path).unwrap();
        let previous = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", path) };
        Self(previous)
    }
}

impl Drop for IsolatedHome {
    fn drop(&mut self) {
        if let Some(previous) = self.0.take() {
            unsafe { std::env::set_var("HOME", previous) };
        } else {
            unsafe { std::env::remove_var("HOME") };
        }
    }
}

fn wait_for(
    room: &agentconductor::ui::Room,
    ready: impl Fn(&serde_json::Value) -> bool,
    label: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        if ready(&snapshot) {
            return;
        }
        assert!(Instant::now() < deadline, "{label} timed out: {snapshot}");
        thread::sleep(Duration::from_millis(25));
    }
}

fn click(context: &egui::Context, probe: &mut Probe, label: &str) -> egui::FullOutput {
    let mut output = frame(context, probe, Vec::new());
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("button {label:?} missing; labels: {:?}", labels(&output)));
    let position = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    assert!(position.y <= 1700.0, "button {label:?} outside test screen");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut pressed = frame(
        context,
        probe,
        vec![
            Event::PointerMoved(position),
            Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    pressed.textures_delta.set.clear();
    pressed.textures_delta.free.clear();
    frame(
        context,
        probe,
        vec![Event::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    )
}

fn frame(context: &egui::Context, probe: &mut Probe, events: Vec<Event>) -> egui::FullOutput {
    frame_with_height(context, probe, events, 1800.0)
}

fn frame_with_height(
    context: &egui::Context,
    probe: &mut Probe,
    events: Vec<Event>,
    height: f32,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1200.0, height))),
            events,
            ..Default::default()
        },
        |ui| probe.update_ui(ui),
    )
}

fn labels(output: &egui::FullOutput) -> Vec<String> {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

fn visible_text(output: &egui::FullOutput) -> String {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}
