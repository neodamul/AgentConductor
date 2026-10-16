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
    let enabled = current && !running && !pending;
    if confirming.is_none() {
        if ui
            .add_enabled(enabled, egui::Button::new("Review execution and approve"))
            .accessible_name("Review exact plan before approving execution")
            .clicked()
        {
            *confirming = Some((revision.into(), digest.into()));
        }
        return None;
    }

    let (confirmed_revision, confirmed_digest) = confirming.clone()?;
    let mut start = false;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.heading("Execution consent · exact reviewed plan");
        ui.label("Approving starts the registered CLI agents. Review their assignments, commands, and writable paths.");
        ui.label(format!("Goal · {}", proposal["plan"]["goal"].as_str().unwrap_or("unknown")));
        ui.label(format!("Repository · {}", snapshot["config"]["repository"].as_str().unwrap_or("unknown")));
        ui.label(format!("Repository base · {}", proposal["repository_base"].as_str().unwrap_or("unknown")));
        ui.label(format!("Workspace mode · {}", proposal["plan"]["workspace_mode"].as_str().unwrap_or("unknown")));
        ui.label(format!("Plan digest · {confirmed_digest}"));
        if let Some(tasks) = proposal["plan"]["tasks"].as_array() {
            for task in tasks {
                let agent = proposal["plan"]["agents"]
                    .as_array()
                    .and_then(|agents| agents.iter().find(|agent| agent["id"] == task["agent"]));
                ui.group(|ui| {
                    ui.strong(format!(
                        "{} · {} · {}",
                        text(task, "id"),
                        text(task, "agent"),
                        task["model"].as_str().unwrap_or("CLI default")
                    ));
                    ui.label(format!("Executable · {}", agent.map(|value| text(value, "executable")).unwrap_or_else(|| "unknown".into())));
                    ui.label(format!("Permission · {}", agent.map(|value| text(value, "permission_mode")).unwrap_or_else(|| "unknown".into())));
                    ui.label(format!("Alternate agent · {}", agent.map(|value| {
                        value["fallback"]["agent"].as_str().filter(|name| !name.is_empty()).unwrap_or("wait for availability").to_owned()
                    }).unwrap_or_else(|| "unknown".into())));
                    ui.label(format!("Writable paths · {}", task["paths"].as_array().map(|paths| paths.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", ")).unwrap_or_default()));
                    ui.label(format!("Arguments · {}", serde_json::to_string(&agent.map(|value| &value["args"]).unwrap_or(&Value::Null)).unwrap_or_default()));
                    ui.label(format!("Test command · {}", serde_json::to_string(&task["test_command"]).unwrap_or_default()));
                });
            }
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(!pending, egui::Button::new("Cancel")).clicked() {
                *confirming = None;
            }
            if ui
                .add_enabled(enabled, egui::Button::new("Confirm and start"))
                .accessible_name("Confirm exact reviewed plan and start agent execution")
                .clicked()
            {
                *confirming = None;
                start = true;
            }
        });
    });
    start.then_some(Event::Start(confirmed_revision, confirmed_digest))
}

fn text(value: &Value, key: &str) -> String {
    value[key]
        .as_str()
        .filter(|text| !text.is_empty())
        .unwrap_or("unknown")
        .into()
}
