//! GitHub #212: `agentconductor io conformance <exe>` checks a CLI against the acio contract with canned inputs.
use super::{Rules, validate_stream};
use serde_json::{Value, json};
use std::{path::Path, process::Stdio, time::Duration};
use tokio::io::AsyncWriteExt;

pub struct Options {
    pub describe_timeout: Duration,
    pub run_timeout: Duration,
}

#[derive(Debug)]
pub enum Outcome {
    Pass,
    Fail(String),
    Skip(String),
}

pub struct Check {
    pub name: String,
    pub outcome: Outcome,
}

pub struct Report {
    pub checks: Vec<Check>,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.checks
            .iter()
            .all(|c| !matches!(c.outcome, Outcome::Fail(_)))
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for check in &self.checks {
            let (mark, note) = match &check.outcome {
                Outcome::Pass => ("pass", String::new()),
                Outcome::Fail(why) => ("FAIL", format!(" - {why}")),
                Outcome::Skip(why) => ("skip", format!(" - {why}")),
            };
            writeln!(f, "{mark:<5}{}{note}", check.name)?;
        }
        let passed = self
            .checks
            .iter()
            .filter(|c| matches!(c.outcome, Outcome::Pass))
            .count();
        write!(f, "{passed} of {} checks passed", self.checks.len())
    }
}

/// Runs the checks in order; a CLI that cannot describe itself skips the rest.
pub async fn run(executable: &Path, options: &Options) -> Report {
    let mut checks = Vec::new();
    let mut add = |name: &str, outcome| {
        checks.push(Check {
            name: name.into(),
            outcome,
        })
    };
    let described = crate::acio::describe(executable, options.describe_timeout).await;
    let (versions, ask) = match &described {
        Ok(document) => {
            add("describe", Outcome::Pass);
            let versions = document["acio"]["versions"].as_array();
            let speaks_v1 = versions.is_some_and(|v| v.iter().any(|n| n == 1));
            add(
                "versions",
                if speaks_v1 {
                    Outcome::Pass
                } else {
                    Outcome::Fail("acio.versions does not list 1".into())
                },
            );
            (true, document["capabilities"]["read_only"] == true)
        }
        Err(why) => {
            add("describe", Outcome::Fail(why.clone()));
            add("versions", Outcome::Skip("needs the description".into()));
            (false, false)
        }
    };
    let ask_outcome = if !versions {
        Outcome::Skip("needs the description".into())
    } else if !ask {
        Outcome::Skip("the CLI does not declare read-only support".into())
    } else {
        ask_run(executable, options.run_timeout).await
    };
    add("ask run", ask_outcome);
    let invalid = if versions {
        invalid_input(executable, options.run_timeout).await
    } else {
        Outcome::Skip("needs the description".into())
    };
    add("invalid input", invalid);
    Report { checks }
}

async fn feed(
    executable: &Path,
    line: &str,
    limit: Duration,
) -> Result<(Option<i32>, String), String> {
    let exe = crate::executable::resolve(&executable.to_string_lossy())
        .unwrap_or_else(|| executable.to_path_buf());
    let mut child = tokio::process::Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("spawn: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    // A CLI that exits without reading closes the pipe; that is its answer, not a harness error.
    let _ = stdin.write_all(format!("{line}\n").as_bytes()).await;
    drop(stdin);
    let output = tokio::time::timeout(limit, child.wait_with_output())
        .await
        .map_err(|_| format!("no exit within {} ms", limit.as_millis()))?
        .map_err(|e| e.to_string())?;
    Ok((
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    ))
}

async fn ask_run(executable: &Path, limit: Duration) -> Outcome {
    let input = json!({
        "schema": "acio.input/1", "request_id": "conformance-1", "mode": "ask", "role": "assistant",
        "instruction": {"text": "Reply with the single word ok. Do not change any file."},
        "workspace": {"root": std::env::temp_dir(), "access": "read_only"},
        "accept": {"versions": [1]}
    });
    match feed(executable, &input.to_string(), limit).await {
        Err(why) => Outcome::Fail(why),
        Ok((code, stdout)) => {
            if let Err(why) = validate_stream("events", &stdout, Rules::default()) {
                return Outcome::Fail(why);
            }
            let status = stdout
                .lines()
                .filter_map(|l| serde_json::from_str::<Value>(l).ok())
                .find(|e| e["type"] == "result")
                .map(|e| e["data"]["status"].clone());
            match (code, status.as_ref().and_then(Value::as_str)) {
                (Some(0), Some("success")) => Outcome::Pass,
                (code, status) => Outcome::Fail(format!(
                    "expected a success result and exit code 0, got {status:?} and {code:?}"
                )),
            }
        }
    }
}

async fn invalid_input(executable: &Path, limit: Duration) -> Outcome {
    match feed(executable, r#"{"not":"acio"}"#, limit).await {
        Err(why) => Outcome::Fail(why),
        Ok((Some(2), _)) => Outcome::Pass,
        Ok((code, _)) => Outcome::Fail(format!(
            "an invalid input should end with exit code 2, got {code:?}"
        )),
    }
}
