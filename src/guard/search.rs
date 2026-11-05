//! Grep and Glob: a search confined to the workspace runs at once, one aimed at credentials or
//! metadata asks a human.
use super::{
    Disposition, Policy,
    spelling::{keys, metadata, sensitive, string},
};
use serde_json::Value;
use std::path::Path;

impl Policy {
    pub(super) fn search(&self, tool: &str, input: &Value) -> Result<Disposition, String> {
        // A search confined to the workspace runs at once; one aimed at credentials
        // or metadata keeps its complete bounded input for a human decision.
        keys(
            input,
            if tool == "Glob" {
                &["pattern", "path"]
            } else {
                &[
                    "pattern",
                    "path",
                    "glob",
                    "type",
                    "output_mode",
                    "-A",
                    "-B",
                    "-C",
                    "-n",
                    "-i",
                    "head_limit",
                    "offset",
                    "multiline",
                ]
            },
        )?;
        let pattern = string(input, "pattern", true)?;
        if tool == "Glob"
            && (pattern.starts_with('/')
                || pattern.contains('\\')
                || pattern.split('/').any(|s| s == ".."))
        {
            return Err("glob patterns cannot escape the workspace".into());
        }
        for key in ["-n", "-i", "multiline"] {
            if input.get(key).is_some_and(|v| !v.is_boolean()) {
                return Err(format!("{key} must be boolean"));
            }
        }
        for key in ["-A", "-B", "-C", "head_limit", "offset"] {
            if input.get(key).is_some_and(|v| v.as_u64().is_none()) {
                return Err(format!("{key} must be a nonnegative integer"));
            }
        }
        for key in ["glob", "type", "output_mode"] {
            if input.get(key).is_some() {
                string(input, key, true)?;
            }
        }
        if input.get("path").is_some() {
            let path = string(input, "path", true)?;
            if path != "."
                && Path::new(path) != self.workspace
                && Path::new(path) != self.configured
            {
                let relative = self.relative(path)?;
                self.inspect(&relative, false, true)?;
                if sensitive(&relative) {
                    return Ok(Disposition::Human);
                }
            }
        }
        let targets = [Some(pattern), input["glob"].as_str()];
        Ok(
            if targets.into_iter().flatten().any(|t| {
                let t = t.to_ascii_lowercase();
                sensitive(&t) || t.contains(".env") || metadata(&t)
            }) {
                Disposition::Human
            } else {
                Disposition::Automatic
            },
        )
    }
}
