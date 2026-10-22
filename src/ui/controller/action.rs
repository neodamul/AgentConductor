use super::*;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionRequest {
    pub(crate) revision: String,
    pub(crate) action: Action,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Action {
    Quota {
        agent: String,
    },
    Approve {
        sha: String,
    },
    Merge {
        sha: String,
    },
    Reject {
        reason: String,
    },
    Revise {
        task: String,
        feedback: String,
    },
    Reassign {
        task: String,
        agent: String,
    },
    Model {
        task: String,
        model: String,
    },
    Schedule {
        task: String,
        at: String,
    },
    ToolApprove {
        sha: String,
    },
    ToolReject {
        sha: String,
    },
    ToolRefresh {
        sha: String,
    },
    ScopeRecovery {
        task: String,
        paths: Vec<String>,
        test: Option<Vec<String>>,
        digest: String,
    },
    Resume {},
    Recover {},
    Stop {
        reason: String,
    },
}

impl Action {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Quota { .. } => "quota",
            Self::Approve { .. } => "approve",
            Self::Merge { .. } => "merge",
            Self::Reject { .. } => "reject",
            Self::Revise { .. } => "revise",
            Self::Reassign { .. } => "reassign",
            Self::Model { .. } => "model",
            Self::Schedule { .. } => "schedule",
            Self::ToolApprove { .. } => "tool_approve",
            Self::ToolReject { .. } => "tool_reject",
            Self::ToolRefresh { .. } => "tool_refresh",
            Self::ScopeRecovery { .. } => "scope_recovery",
            Self::Resume {} => "resume",
            Self::Recover {} => "recover",
            Self::Stop { .. } => "stop",
        }
    }

    pub(crate) fn args(&self, report: &Report, dir: &Path) -> Result<Vec<String>, String> {
        Ok(match self {
            Self::Quota { agent } => {
                if !report
                    .plan
                    .agents
                    .iter()
                    .any(|profile| &profile.id == agent)
                {
                    return Err(crate::ui::messages::text("native.unknown_quota_agent").into());
                }
                vec!["quota-refresh".into(), agent.clone()]
            }
            Self::Approve { sha } => vec!["approve".into(), sha.clone()],
            Self::Merge { sha } => {
                if report.state != "approved"
                    || report
                        .candidate
                        .as_ref()
                        .is_none_or(|candidate| &candidate.commit != sha)
                {
                    return Err(crate::ui::messages::text(
                        "native.merge_requires_the_exact_approved_candidate",
                    )
                    .into());
                }
                vec!["merge".into()]
            }
            Self::Reject { reason } => vec!["reject".into(), reason.clone()],
            Self::Revise { task, feedback } => {
                vec!["revise".into(), task.clone(), feedback.clone()]
            }
            Self::Reassign { task, agent } => vec!["reassign".into(), task.clone(), agent.clone()],
            Self::Model { task, model } => {
                vec!["session-model".into(), task.clone(), model.clone()]
            }
            Self::Schedule { task, at } => vec!["schedule".into(), task.clone(), at.clone()],
            Self::ToolApprove { sha } => vec!["tool-approve".into(), sha.clone()],
            Self::ToolReject { sha } => vec!["tool-reject".into(), sha.clone()],
            Self::ToolRefresh { sha } => vec!["tool-refresh".into(), sha.clone()],
            Self::ScopeRecovery {
                task,
                paths,
                test,
                digest,
            } => scope_recovery_args(task, paths, test, digest)?,
            Self::Resume {} if report.state == "stopped" => {
                vec!["ui-resume".into(), workflow::stop_digest(dir)?]
            }
            Self::Resume {} if report.state == "failed" && !workflow::repairable(report) => {
                return Err(crate::ui::messages::text(
                    "native.failed_run_cannot_be_resumed_revise_or_reassign",
                )
                .into());
            }
            Self::Resume {} => vec!["resume-due".into()],
            Self::Recover {} => vec!["recover".into()],
            Self::Stop { .. } => {
                return Err(crate::ui::messages::text("native.stop_is_handled_directly").into());
            }
        })
    }
}

fn scope_recovery_args(
    task: &str,
    paths: &[String],
    test: &Option<Vec<String>>,
    digest: &str,
) -> Result<Vec<String>, String> {
    let paths = serde_json::to_string(paths).map_err(|error| error.to_string())?;
    let test = serde_json::to_string(test).map_err(|error| error.to_string())?;
    if task.len() + paths.len() + test.len() + digest.len() > 8 * 1024 {
        return Err(crate::ui::messages::text("native.scope_recovery_request_too_large").into());
    }
    Ok(vec![
        "scope-recovery".into(),
        task.into(),
        paths,
        test,
        digest.into(),
    ])
}
