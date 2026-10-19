use serde_json::Value;

use super::State;

pub fn evidence_request() -> &'static [u8] {
    br#"{"kind":"evidence"}"#
}

pub fn evidence_reply(
    state: &mut State,
    current: Option<&Value>,
    evidence: Value,
) -> Result<(), String> {
    let expected = current
        .and_then(|snapshot| snapshot["revision"].as_str())
        .ok_or("current run revision is unavailable")?;
    if evidence["revision"].as_str() != Some(expected) {
        return Err("run changed while loading review evidence; refresh and try again".into());
    }
    state.diff = evidence["diff"].as_str().unwrap_or_default().to_owned();
    state.evidence = Some(evidence);
    Ok(())
}

pub fn stop_request(snapshot: &Value, reason: &str) -> Result<Vec<u8>, String> {
    if reason.trim().is_empty() || reason.len() > 16 * 1024 {
        return Err("stop needs a nonempty reason up to 16 KiB".into());
    }
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("current run revision is unavailable")?;
    serde_json::to_vec(&serde_json::json!({
        "kind":"action",
        "revision":revision,
        "action":{"kind":"stop","reason":reason}
    }))
    .map_err(|error| error.to_string())
}

pub(in crate::native_ui_probe) fn action_reply(
    worker: &super::super::RequestWorker,
    response: &Value,
) -> Result<Option<String>, String> {
    if response["accepted"] != true {
        return Err(response["message"]
            .as_str()
            .unwrap_or("stop request was not accepted")
            .to_owned());
    }
    worker.send(br#"{"kind":"snapshot"}"#)?;
    Ok(response["message"].as_str().map(str::to_owned))
}

impl super::super::Probe {
    pub(in crate::native_ui_probe) fn decide_room(
        &mut self,
        decision: super::CandidateDecision,
        reason: String,
    ) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Agent Room snapshot is unavailable".to_owned())
            .and_then(|snapshot| super::candidate_decision_request(snapshot, decision, &reason))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Agent Room controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(super::super::Pending::Action);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(in crate::native_ui_probe) fn load_evidence(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
            .and_then(|worker| worker.send(evidence_request()));
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(super::super::Pending::Evidence);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(in crate::native_ui_probe) fn refresh_room(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot"}"#));
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(super::super::Pending::Refresh);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(in crate::native_ui_probe) fn stop_room(&mut self, reason: String) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Agent Room snapshot is unavailable".to_owned())
            .and_then(|snapshot| stop_request(snapshot, &reason))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Agent Room controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(super::super::Pending::Action);
            }
            Err(error) => self.error = Some(error),
        }
    }
}
