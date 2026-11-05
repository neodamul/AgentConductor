mod command;
mod forms;
mod identity;
mod search;
mod spelling;

#[cfg(windows)]
pub(crate) use identity::windows_identity;
use identity::{directory_identity, file_metadata};
use serde_json::Value;
use spelling::{keys, safe_relative, sensitive, string};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Disposition {
    Automatic,
    Human,
    /// Refuse this call only; the worker continues and may choose another action.
    Deny(String),
}

pub struct Policy {
    workspace: PathBuf,
    configured: PathBuf,
    scopes: Vec<String>,
    identity: (u64, u64),
}

impl Policy {
    pub fn file_target(&self, tool: &str, input: &Value) -> Result<Option<PathBuf>, String> {
        self.classify(tool, input)?;
        if matches!(tool, "Read" | "Edit" | "Write") {
            Ok(Some(self.workspace.join(self.relative(string(
                input,
                "file_path",
                true,
            )?)?)))
        } else {
            Ok(None)
        }
    }
    pub fn evaluator(
        self,
        mut human: crate::claude_control::Evaluator,
    ) -> crate::claude_control::Evaluator {
        Box::new(move |tool, input| match self.classify(tool, input)? {
            Disposition::Automatic => Ok(crate::claude_control::ToolDecision {
                allow: true,
                reason: "Workspace file policy permits the bounded input".into(),
            }),
            Disposition::Human => human(tool, input),
            Disposition::Deny(reason) => Ok(crate::claude_control::ToolDecision::deny(&reason)),
        })
    }
    pub fn new(workspace: &Path, scopes: &[String]) -> Result<Self, String> {
        if !workspace.is_absolute() || scopes.is_empty() || scopes.len() > 128 {
            return Err("file policy needs an absolute workspace and bounded owned scopes".into());
        }
        file_metadata(workspace, false)?;
        for scope in scopes {
            safe_relative(scope)?;
        }
        Ok(Self {
            workspace: fs::canonicalize(workspace).map_err(|e| e.to_string())?,
            configured: workspace.into(),
            scopes: scopes.into(),
            identity: directory_identity(workspace)?,
        })
    }

    fn validate_workspace(&self) -> Result<(), String> {
        file_metadata(&self.configured, false)?;
        if fs::canonicalize(&self.configured).map_err(|e| e.to_string())? != self.workspace
            || directory_identity(&self.configured)? != self.identity
        {
            return Err("configured workspace identity changed".into());
        }
        Ok(())
    }

    fn relative(&self, path: &str) -> Result<String, String> {
        #[cfg(windows)]
        if let Some(relative) = self.windows_relative(path)? {
            safe_relative(&relative)?;
            return Ok(relative);
        }
        // Validate spelling before Path normalizes repeated separators or dot components.
        let relative = if path.starts_with('/') {
            let relative = path
                .strip_prefix(self.workspace.to_str().ok_or("non-UTF8 workspace")?)
                .or_else(|| path.strip_prefix(self.configured.to_str().unwrap_or("")))
                .and_then(|s| s.strip_prefix('/'))
                .ok_or("tool path is outside the workspace")?;
            relative.to_owned()
        } else {
            path.to_owned()
        };
        safe_relative(&relative)?;
        Ok(relative)
    }

    /// Windows tools send `C:\…`, `C:/…` or `\\?\C:\…` paths and may use `\` inside the
    /// workspace. `\` is a separator here, drive and folder names compare without case,
    /// and the remainder is checked by the same relative-path rules as on Unix.
    #[cfg(windows)]
    fn windows_relative(&self, path: &str) -> Result<Option<String>, String> {
        let plain = |p: &str| {
            let p = p.replace('\\', "/");
            p.strip_prefix("//?/").map(str::to_owned).unwrap_or(p)
        };
        let spelled = plain(path);
        let bytes = spelled.as_bytes();
        let absolute = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
        if !absolute {
            // Relative spellings only change separators; rooted forms stay for the Unix check.
            return Ok((!path.starts_with(['/', '\\']) && path.contains('\\')).then_some(spelled));
        }
        for root in [&self.workspace, &self.configured] {
            let root = plain(root.to_str().ok_or("non-UTF8 workspace")?);
            let root = root.trim_end_matches('/');
            if spelled.len() > root.len()
                && spelled.is_char_boundary(root.len())
                && spelled[..root.len()].eq_ignore_ascii_case(root)
                && let Some(tail) = spelled[root.len()..].strip_prefix('/')
            {
                return Ok(Some(tail.to_owned()));
            }
        }
        Err("tool path is outside the workspace".into())
    }

    fn inspect(&self, relative: &str, create: bool, directory: bool) -> Result<(), String> {
        let mut current = self.workspace.clone();
        let parts: Vec<_> = relative.split('/').collect();
        for (index, part) in parts.iter().enumerate() {
            current.push(part);
            match fs::symlink_metadata(&current) {
                Ok(meta) => file_metadata(
                    &current,
                    index + 1 == parts.len() && !(directory && meta.is_dir()),
                )?,
                Err(e) if create && e.kind() == std::io::ErrorKind::NotFound => break,
                Err(e) => return Err(format!("tool path cannot be inspected: {e}")),
            }
        }
        Ok(())
    }

    fn file(&self, input: &Value, write: bool, create: bool) -> Result<Disposition, String> {
        let relative = self.relative(string(input, "file_path", true)?)?;
        if write
            && !self.scopes.iter().any(|scope| {
                crate::plan::overlaps(&relative, scope) && (relative.len() >= scope.len())
            })
        {
            return Err("tool write is outside the task's owned scopes".into());
        }
        // Existing ancestors are still inspected, so symlinked parents remain errors.
        self.inspect(&relative, true, false)?;
        if !create
            && let Err(e) = fs::symlink_metadata(self.workspace.join(&relative))
            && e.kind() == std::io::ErrorKind::NotFound
        {
            // A missing file exposes no content; refusing keeps the worker alive.
            return Ok(Disposition::Deny(format!("{relative} does not exist")));
        }
        self.inspect(&relative, create, false)?;
        Ok(if sensitive(&relative) {
            Disposition::Human
        } else {
            Disposition::Automatic
        })
    }

    pub fn classify(&self, tool: &str, input: &Value) -> Result<Disposition, String> {
        self.validate_workspace()?;
        if serde_json::to_vec(input).map_err(|e| e.to_string())?.len() > 64 * 1024 {
            return Err("tool input exceeds 64 KiB".into());
        }
        match tool {
            "Read" => {
                keys(input, &["file_path", "offset", "limit", "pages"])?;
                for key in ["offset", "limit"] {
                    if input
                        .get(key)
                        .is_some_and(|v| v.as_u64().is_none_or(|n| n == 0))
                    {
                        return Err(format!("{key} must be a positive integer"));
                    }
                }
                if input.get("pages").is_some() {
                    string(input, "pages", true)?;
                }
                self.file(input, false, false)
            }
            "Write" => {
                keys(input, &["file_path", "content"])?;
                string(input, "content", false)?;
                self.file(input, true, true)
            }
            "Edit" => {
                keys(
                    input,
                    &["file_path", "old_string", "new_string", "replace_all"],
                )?;
                string(input, "old_string", false)?;
                string(input, "new_string", false)?;
                if input.get("replace_all").is_some_and(|v| !v.is_boolean()) {
                    return Err("replace_all must be boolean".into());
                }
                self.file(input, true, false)
            }
            "Bash" => {
                keys(
                    input,
                    &["command", "description", "timeout", "run_in_background"],
                )?;
                let command = string(input, "command", true)?;
                if input.get("description").is_some() {
                    string(input, "description", true)?;
                }
                if input
                    .get("run_in_background")
                    .is_some_and(|v| v != &Value::Bool(false))
                {
                    return Err("background commands are outside the approval lifecycle".into());
                }
                if input
                    .get("timeout")
                    .is_some_and(|v| v.as_u64().is_none_or(|n| n == 0 || n > 600_000))
                {
                    return Err("command timeout must be 1..600000 milliseconds".into());
                }
                Ok(self.command(command))
            }
            "Grep" | "Glob" => self.search(tool, input),
            _ => Err("unsupported native tool; execution refused".into()),
        }
    }
}
