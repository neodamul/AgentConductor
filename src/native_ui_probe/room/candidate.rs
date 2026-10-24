use super::CandidateDecision;
use serde_json::Value;

pub fn candidate_decision_request(
    snapshot: &Value,
    decision: CandidateDecision,
    reason: &str,
) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("current run revision is unavailable")?;
    let state = snapshot["report"]["state"]
        .as_str()
        .ok_or("current run state is unavailable")?;
    let candidate = snapshot["report"]["candidate"]["commit"]
        .as_str()
        .filter(|commit| !commit.is_empty())
        .ok_or("no candidate is available")?;
    let action = match decision {
        CandidateDecision::Approve if state == "waiting_approval" => {
            serde_json::json!({"kind":"approve","sha":candidate})
        }
        CandidateDecision::Approve => return Err("candidate is not waiting for approval".into()),
        CandidateDecision::Merge if state == "approved" => {
            serde_json::json!({"kind":"merge","sha":candidate})
        }
        CandidateDecision::Merge => return Err("candidate must be approved before merge".into()),
        CandidateDecision::Reject if !reason.trim().is_empty() && reason.len() <= 16 * 1024 => {
            serde_json::json!({"kind":"reject","reason":reason})
        }
        CandidateDecision::Reject => {
            return Err("rejection needs a nonempty reason up to 16 KiB".into());
        }
    };
    serde_json::to_vec(&serde_json::json!({
        "kind":"action", "revision":revision, "action":action
    }))
    .map_err(|error| error.to_string())
}
