use super::*;
use crate::{
    adapters::InputMode,
    process::{self, ProcessSpec},
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

struct RunOptions<'a> {
    model: Option<&'a str>,
    timeout: Duration,
}

pub async fn run_json(
    profile: &AgentProfile,
    cwd: &Path,
    dir: &Path,
    prompt: &str,
    field: &str,
    stop_file: Option<PathBuf>,
) -> Result<Value, String> {
    run_json_within(
        profile,
        cwd,
        dir,
        prompt,
        field,
        stop_file,
        Duration::from_secs(1800),
    )
    .await
}

pub async fn run_json_with_model(
    profile: &AgentProfile,
    model: Option<&str>,
    cwd: &Path,
    dir: &Path,
    prompt: &str,
    field: &str,
    stop_file: Option<PathBuf>,
) -> Result<Value, String> {
    run_json_within_model(
        profile,
        cwd,
        dir,
        prompt,
        field,
        stop_file,
        RunOptions {
            model,
            timeout: Duration::from_secs(1800),
        },
    )
    .await
}

pub async fn run_json_within(
    profile: &AgentProfile,
    cwd: &Path,
    dir: &Path,
    prompt: &str,
    field: &str,
    stop_file: Option<PathBuf>,
    timeout: Duration,
) -> Result<Value, String> {
    run_json_within_model(
        profile,
        cwd,
        dir,
        prompt,
        field,
        stop_file,
        RunOptions {
            model: None,
            timeout,
        },
    )
    .await
}

async fn run_json_within_model(
    profile: &AgentProfile,
    cwd: &Path,
    dir: &Path,
    prompt: &str,
    field: &str,
    stop_file: Option<PathBuf>,
    options: RunOptions<'_>,
) -> Result<Value, String> {
    if prompt.len() > 128 * 1024 {
        return Err("coordinator prompt exceeds 128 KiB".into());
    }
    let task_file = dir.join("prompt.txt");
    let args = read_only_args_with_model(
        profile,
        options.model,
        task_file
            .to_str()
            .ok_or("non-UTF8 coordinator context path")?,
    )?;
    let args = if !cwd.join(".git").exists() {
        adapters::direct_folder_args(profile.parser, args)?
    } else {
        args
    };
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    let mut prompt = prompt.to_owned();
    fs::write(&task_file, &prompt).map_err(|e| e.to_string())?;
    let mut asked_again = false;
    // A model "at capacity" is momentary: keep each bounded retry's logs.
    let mut retries = 0;
    loop {
        let result = process::run_until(
            ProcessSpec {
                executable: profile.executable.clone().into(),
                args: args.clone(),
                cwd: cwd.into(),
                input: if profile.input_mode == InputMode::Stdin {
                    prompt.as_bytes().to_vec()
                } else {
                    vec![]
                },
                log_dir: dir.join("process"),
                timeout: options.timeout,
                signal_grace: Duration::from_secs(5),
                environment: profile.environment.clone(),
            },
            stop_file.clone(),
        )
        .await?;
        let outcome = if result.exit_code != Some(0) || result.timed_out || result.stopped {
            Err(failure("coordinator", &result))
        } else {
            read_profile_document(profile, &result.stdout, field)
        };
        match outcome {
            Err(error)
                if retries < 3
                    && !result.stopped
                    && crate::availability::transient_capacity(&error) =>
            {
                retries += 1;
                fs::rename(
                    dir.join("process"),
                    dir.join(format!("process-capacity-{retries}")),
                )
                .map_err(|e| e.to_string())?;
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
            Err(error)
                if !asked_again
                    && result.exit_code == Some(0)
                    && !result.timed_out
                    && !result.stopped
                    && error.starts_with(document::NO_JSON) =>
            {
                asked_again = true;
                prompt = crate::prompt_form::with_previous_reply(&prompt, &error);
                fs::rename(dir.join("process"), dir.join("process-format-1"))
                    .map_err(|e| e.to_string())?;
                fs::write(&task_file, &prompt).map_err(|e| e.to_string())?;
            }
            outcome => return outcome,
        }
    }
}
