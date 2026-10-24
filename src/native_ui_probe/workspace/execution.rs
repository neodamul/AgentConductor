use eframe::egui;
use serde_json::Value;

pub enum Event {
    Start(String, String),
}

pub fn start_request(
    snapshot: &Value,
    confirmed_revision: &str,
    confirmed_digest: &str,
) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("workspace revision is unavailable")?;
    let proposal = &snapshot["proposal"];
    if revision.is_empty()
        || confirmed_digest.is_empty()
        || revision != confirmed_revision
        || proposal["phase"] != "ready"
        || proposal["digest"].as_str() != Some(confirmed_digest)
        || proposal["plan"]["tasks"]
            .as_array()
            .is_none_or(Vec::is_empty)
    {
        return Err("the reviewed plan changed or is no longer ready".into());
    }
    serde_json::to_vec(&serde_json::json!({
        "kind":"start", "revision":revision, "digest":confirmed_digest
    }))
    .map_err(|error| error.to_string())
}

pub fn execution_ready(snapshot: &Value, expected_run: &str) -> bool {
    !expected_run.is_empty()
        && snapshot["run_available"] == true
        && snapshot["proposal"]["run"].as_str() == Some(expected_run)
}

pub fn controls(
    snapshot: &Value,
    current: bool,
    running: bool,
    pending: bool,
    confirming: &mut Option<(String, String)>,
    ui: &mut egui::Ui,
) {
    controls_in_language(snapshot, current, running, pending, confirming, "en", ui)
}

pub fn controls_in_language(
    snapshot: &Value,
    current: bool,
    running: bool,
    pending: bool,
    confirming: &mut Option<(String, String)>,
    language: &str,
    ui: &mut egui::Ui,
) {
    let proposal = &snapshot["proposal"];
    if proposal["phase"] != "ready" {
        *confirming = None;
        return;
    }
    let revision = snapshot["revision"].as_str().unwrap_or_default();
    let digest = proposal["digest"].as_str().unwrap_or_default();
    if confirming
        .as_ref()
        .is_some_and(|(rev, hash)| rev != revision || hash != digest)
    {
        *confirming = None;
    }
    let enabled = current && !running && !pending;
    let review = ui
        .add_enabled(
            enabled && confirming.is_none(),
            egui::Button::new(text(language, "startup.execution_review")),
        )
        .accessible_name(text(language, "startup.execution_review_accessible"));
    if confirming.is_none() && review.clicked() {
        *confirming = Some((revision.into(), digest.into()));
    }
}

pub fn consent_in_language(
    snapshot: &Value,
    enabled: bool,
    pending: bool,
    confirming: &mut Option<(String, String)>,
    language: &str,
    context: &egui::Context,
) -> Option<Event> {
    let proposal = &snapshot["proposal"];
    if proposal["phase"] != "ready" {
        *confirming = None;
        return None;
    }
    let revision = snapshot["revision"].as_str().unwrap_or_default();
    let digest = proposal["digest"].as_str().unwrap_or_default();
    if confirming
        .as_ref()
        .is_some_and(|(rev, hash)| rev != revision || hash != digest)
    {
        *confirming = None;
    }
    let (confirmed_revision, confirmed_digest) = confirming.clone()?;
    let mut start = false;
    let mut open = true;
    egui::Window::new(text(language, "startup.execution_heading"))
        .id(egui::Id::NULL.with("native-execution-consent"))
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .fixed_size(egui::vec2(560.0, 580.0))
        .show(context, |ui| {
            egui::ScrollArea::vertical()
                .max_height(440.0)
                .show(ui, |ui| {
                    ui.label(text(language, "startup.execution_warning"));
                    show_value(
                        ui,
                        language,
                        "startup.execution_goal",
                        &field(&proposal["plan"], "goal", language),
                    );
                    show_value(
                        ui,
                        language,
                        "startup.execution_repository",
                        &field(&snapshot["config"], "repository", language),
                    );
                    show_value(
                        ui,
                        language,
                        "startup.execution_repository_base",
                        &field(proposal, "repository_base", language),
                    );
                    show_value(
                        ui,
                        language,
                        "startup.execution_workspace_mode",
                        &field(&proposal["plan"], "workspace_mode", language),
                    );
                    show_value(
                        ui,
                        language,
                        "startup.execution_plan_digest",
                        &confirmed_digest,
                    );
                    if let Some(tasks) = proposal["plan"]["tasks"].as_array() {
                        for task in tasks {
                            let agent = proposal["plan"]["agents"].as_array().and_then(|agents| {
                                agents.iter().find(|agent| agent["id"] == task["agent"])
                            });
                            ui.group(|ui| {
                                let task_id = field(task, "id", language);
                                let agent_id = field(task, "agent", language);
                                let model = task["model"]
                                    .as_str()
                                    .unwrap_or(text(language, "startup.execution_cli_default"));
                                ui.strong(crate::ui::messages::interpolate(
                                    text(language, "startup.execution_task"),
                                    &[
                                        ("value1", &task_id),
                                        ("value2", &agent_id),
                                        ("value3", model),
                                    ],
                                ));
                                let unavailable = text(language, "startup.execution_unavailable");
                                let executable = agent
                                    .map(|value| field(value, "executable", language))
                                    .unwrap_or_else(|| unavailable.into());
                                let permission = agent
                                    .map(|value| field(value, "permission_mode", language))
                                    .unwrap_or_else(|| unavailable.into());
                                show_value(
                                    ui,
                                    language,
                                    "startup.execution_executable",
                                    &executable,
                                );
                                show_value(
                                    ui,
                                    language,
                                    "startup.execution_permission",
                                    &permission,
                                );
                                let alternate = agent
                                    .and_then(|value| value["fallback"]["agent"].as_str())
                                    .filter(|name| !name.is_empty())
                                    .map(str::to_owned)
                                    .or_else(|| {
                                        agent.map(|_| {
                                            text(
                                                language,
                                                "startup.execution_wait_for_availability",
                                            )
                                            .into()
                                        })
                                    })
                                    .unwrap_or_else(|| unavailable.into());
                                show_value(
                                    ui,
                                    language,
                                    "startup.execution_alternate_agent",
                                    &alternate,
                                );
                                let paths = task["paths"]
                                    .as_array()
                                    .map(|paths| {
                                        paths
                                            .iter()
                                            .filter_map(Value::as_str)
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    })
                                    .unwrap_or_default();
                                show_value(
                                    ui,
                                    language,
                                    "startup.execution_writable_paths",
                                    &paths,
                                );
                                let args = serde_json::to_string(
                                    &agent.map(|value| &value["args"]).unwrap_or(&Value::Null),
                                )
                                .unwrap_or_default();
                                show_value(ui, language, "startup.execution_arguments", &args);
                                let test_command = serde_json::to_string(&task["test_command"])
                                    .unwrap_or_default();
                                show_value(
                                    ui,
                                    language,
                                    "startup.execution_test_command",
                                    &test_command,
                                );
                            });
                        }
                    }
                });
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !pending,
                        egui::Button::new(text(language, "startup.execution_cancel")),
                    )
                    .clicked()
                {
                    *confirming = None;
                }
                let confirm = ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(text(language, "startup.execution_confirm_start")),
                    )
                    .accessible_name(text(language, "startup.execution_confirm_accessible"));
                if confirm.clicked() {
                    *confirming = None;
                    start = true;
                }
            });
        });
    if !open {
        *confirming = None;
    }
    start.then_some(Event::Start(confirmed_revision, confirmed_digest))
}

fn field(value: &Value, key: &str, language: &str) -> String {
    value[key]
        .as_str()
        .filter(|text| !text.is_empty())
        .unwrap_or(text(language, "startup.execution_unavailable"))
        .into()
}

fn show_value(ui: &mut egui::Ui, language: &str, id: &'static str, value: &str) {
    ui.label(crate::ui::messages::interpolate(
        text(language, id),
        &[("value1", value)],
    ));
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
