use super::*;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::ui) struct ActionRequest {
    pub(in crate::ui) revision: String,
    pub(in crate::ui) action: Action,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(in crate::ui) enum Action {
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
        #[serde(default)]
        register: bool,
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
    Resume {},
    Recover {},
    Stop {
        reason: String,
    },
    CoordinatorRequest {
        message: String,
    },
    ScopeRecovery {
        task: String,
        paths: Vec<String>,
        #[serde(default)]
        test_command: Option<Vec<String>>,
        digest: String,
    },
}
impl Action {
    pub(in crate::ui) fn label(&self) -> &'static str {
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
            Self::Resume {} => "resume",
            Self::Recover {} => "recover",
            Self::Stop { .. } => "stop",
            Self::CoordinatorRequest { .. } => "coordinator_request",
            Self::ScopeRecovery { .. } => "scope_recovery",
        }
    }
    pub(in crate::ui) fn args(&self, report: &Report, dir: &Path) -> Result<Vec<String>, String> {
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
                    || report.candidate.as_ref().is_none_or(|c| &c.commit != sha)
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
            Self::Model {
                task,
                model,
                register,
            } => {
                let mut args = vec!["session-model".into(), task.clone(), model.clone()];
                if *register {
                    args.push("--register".into());
                }
                args
            }
            Self::Schedule { task, at } => vec!["schedule".into(), task.clone(), at.clone()],
            Self::ToolApprove { sha } => vec!["tool-approve".into(), sha.clone()],
            Self::ToolReject { sha } => vec!["tool-reject".into(), sha.clone()],
            Self::ToolRefresh { sha } => vec!["tool-refresh".into(), sha.clone()],
            Self::Resume {} if report.state == "stopped" => {
                vec!["ui-resume".into(), workflow::stop_digest(dir)?]
            }
            Self::Resume {} if report.state == "failed" => {
                if report
                    .error
                    .as_deref()
                    .is_some_and(|e| e.starts_with("out-of-scope change:"))
                {
                    return Err(
                        crate::ui::messages::text("native.scope_failure_use_recovery").into(),
                    );
                }
                // GitHub #625: any other failure is retried by Resume (failed tasks, then verification and repair).
                vec!["resume-due".into()]
            }
            Self::Resume {} => {
                // GitHub #637: say what blocks a Resume that would start nothing.
                if let Some((task, status)) = workflow::resume_blocker(dir, report)? {
                    return Err(if task.is_empty() {
                        crate::ui::messages::render(
                            "native.nothing_to_resume_run",
                            &[("value1", &status)],
                        )
                    } else {
                        crate::ui::messages::render(
                            "native.nothing_to_resume_task",
                            &[("value1", &task), ("value2", &status)],
                        )
                    });
                }
                vec!["resume-due".into()]
            }
            Self::Recover {} => vec!["recover".into()],
            Self::ScopeRecovery {
                task,
                paths,
                test_command,
                digest,
            } => workflow::scope_recovery_args(task, paths, test_command.as_deref(), digest)?,
            Self::CoordinatorRequest { message } => {
                vec!["coordinator-request".into(), message.clone()]
            }
            Self::Stop { .. } => {
                return Err(crate::ui::messages::text("native.stop_is_handled_directly").into());
            }
        })
    }
}
