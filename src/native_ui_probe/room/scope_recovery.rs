use serde_json::Value;

pub(super) fn failure_request() -> &'static [u8] {
    br#"{"kind":"scope_failure"}"#
}

pub(super) fn preview_request(
    task: &str,
    paths: &[String],
    test: Option<&[String]>,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({
        "kind":"scope_recovery_preview",
        "task":task,
        "paths":paths,
        "test":test,
    }))
    .map_err(|error| error.to_string())
}

pub(super) fn apply_request(
    snapshot: &Value,
    task: &str,
    paths: &[String],
    test: Option<&[String]>,
    digest: &str,
) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("current run revision is unavailable")?;
    serde_json::to_vec(&serde_json::json!({
        "kind":"action",
        "revision":revision,
        "action":{
            "kind":"scope_recovery",
            "task":task,
            "paths":paths,
            "test":test,
            "digest":digest,
        }
    }))
    .map_err(|error| error.to_string())
}

impl super::super::Probe {
    pub(crate) fn load_scope_failure(&mut self) {
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
            .and_then(|worker| worker.send(failure_request()))
        {
            Ok(()) => {
                self.pending = Some(super::super::Pending::ScopeFailure);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn preview_scope_recovery(
        &mut self,
        task: String,
        paths: Vec<String>,
        test: Option<Vec<String>>,
    ) {
        let request = preview_request(&task, &paths, test.as_deref());
        match request.and_then(|packet| {
            self.worker
                .as_ref()
                .ok_or_else(|| "Agent Room controller is unavailable".to_owned())?
                .send(&packet)
        }) {
            Ok(()) => {
                self.pending = Some(super::super::Pending::ScopeRecoveryPreview);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn apply_scope_recovery(
        &mut self,
        task: String,
        paths: Vec<String>,
        test: Option<Vec<String>>,
        digest: String,
    ) {
        let request = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Agent Room snapshot is unavailable".to_owned())
            .and_then(|snapshot| apply_request(snapshot, &task, &paths, test.as_deref(), &digest))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Agent Room controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match request {
            Ok(()) => {
                if let Some(view) = self.room_view.as_mut() {
                    view.scope_recovery_open = false;
                    view.scope_preview = None;
                }
                self.resume_after_task_feedback = false;
                self.resume_after_tool_approval = false;
                self.pending = Some(super::super::Pending::ScopeRecoveryApply);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(crate) fn apply_scope_failure(&mut self, value: Value) {
        let Some(view) = self.room_view.as_mut() else {
            self.pending = None;
            self.error = Some("Agent Room is unavailable".into());
            return;
        };
        view.scope_task = value["tasks"]
            .as_array()
            .and_then(|tasks| tasks.first())
            .and_then(|task| task["id"].as_str())
            .unwrap_or_default()
            .to_owned();
        let selected = value["tasks"].as_array().and_then(|tasks| tasks.first());
        view.scope_paths = selected
            .and_then(|task| task["paths"].as_array())
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        view.scope_test_command = selected
            .map(|task| {
                serde_json::to_string(&task["test_command"]).unwrap_or_else(|_| "[]".into())
            })
            .unwrap_or_else(|| "[]".into());
        view.scope_preview = None;
        view.scope_failure = Some(value);
        view.scope_recovery_open = true;
        self.pending = None;
        self.error = None;
    }

    pub(crate) fn apply_scope_preview(&mut self, value: Value) {
        if let Some(view) = self.room_view.as_mut() {
            view.scope_preview = Some(value);
            self.pending = None;
            self.error = None;
        } else {
            self.pending = None;
            self.error = Some("Agent Room is unavailable".into());
        }
    }
}
