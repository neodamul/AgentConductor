//! #1027: Report a bug zips the chosen log tails into Downloads, since an issue draft URL cannot carry files.
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

/// Each log keeps only its end, so the zip stays far below GitHub's 25 MB attachment limit.
pub const TAIL: u64 = 256 * 1024;
pub const ENVIRONMENT: &str = "environment";
const MASK: &str = "[masked]";

struct Source {
    id: &'static str,
    name: &'static str,
    path: Option<PathBuf>,
}

/// The newest run's folder under the desktop directory, where its process logs live.
fn latest_run(desktop: &Path) -> Option<PathBuf> {
    fs::read_dir(desktop)
        .ok()?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("coordinator-ui-")
        })
        .filter(|e| e.path().is_dir())
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
}

/// `app` is the `.agentconductor` folder: it holds the request log and the desktop runs.
fn sources(app: &Path) -> Vec<Source> {
    let run = latest_run(&app.join("desktop"));
    vec![
        Source {
            id: ENVIRONMENT,
            name: "environment.txt",
            path: None,
        },
        Source {
            id: "requests",
            name: "request-diagnostics.log",
            path: Some(app.join("request-diagnostics.log")),
        },
        Source {
            id: "messenger",
            name: "messenger.log",
            path: run
                .as_deref()
                .map(|run| crate::messenger::sibling(run, "messenger.log")),
        },
        Source {
            id: "run",
            name: "stderr.log",
            path: run.map(|run| run.join("process/stderr.log")),
        },
    ]
}

fn environment() -> String {
    format!(
        "AgentConductor: {}\nOS: {}\nArchitecture: {}\n",
        crate::update::current(),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

/// The files the operator can choose, with the size each would add to the zip.
pub fn files(app: &Path) -> Value {
    let files: Vec<Value> = sources(app)
        .into_iter()
        .map(|s| {
            let bytes = match &s.path {
                _ if s.id == ENVIRONMENT => Some(environment().len() as u64),
                Some(path) => fs::metadata(path).ok().map(|m| m.len().min(TAIL)),
                None => None,
            };
            json!({"id":s.id,"name":s.name,"present":bytes.is_some(),"bytes":bytes.unwrap_or(0)})
        })
        .collect();
    json!({ "files": files })
}

fn read_tail(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    file.seek(SeekFrom::Start(len.saturating_sub(TAIL))).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

const SECRET_PREFIXES: [&str; 9] = [
    "sk-",
    "ghp_",
    "gho_",
    "ghs_",
    "github_pat_",
    "glpat-",
    "xoxb-",
    "xoxp-",
    "AKIA",
];
const SECRET_KEYS: [&str; 6] = [
    "token",
    "secret",
    "password",
    "apikey",
    "api_key",
    "authorization",
];

fn secret_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase().replace('-', "_");
    SECRET_KEYS.iter().any(|k| key.contains(k))
}

/// Masks one word; `follows_key` is set when the previous word named a secret (`Bearer`, `"token":`).
fn mask_word(word: &str, follows_key: bool) -> (String, bool) {
    let lower = word.to_ascii_lowercase();
    if follows_key {
        // `"token": "x"` splits into `token`, `:` and `x`; `Authorization: Bearer x` masks x.
        if matches!(word, ":" | "=") || lower == "bearer" {
            return (word.into(), true);
        }
        return (MASK.into(), false);
    }
    if lower.contains("://") && lower.contains("hook") {
        return (MASK.into(), false);
    }
    if word.len() >= 16 && SECRET_PREFIXES.iter().any(|p| word.starts_with(p)) {
        return (MASK.into(), false);
    }
    if let Some(at) = word.find(['=', ':'])
        && !lower.contains("://")
    {
        let (key, value) = word.split_at(at);
        if secret_key(key) {
            return if value.len() > 1 {
                (format!("{key}{}{MASK}", &value[..1]), false)
            } else {
                (word.into(), true)
            };
        }
    }
    (word.into(), lower == "bearer" || secret_key(&lower))
}

/// Masks webhook URLs, well-known token shapes and values named token/secret/password/key.
pub fn mask(text: &str) -> String {
    let separator = |c: char| c.is_whitespace() || "\"'`,;<>()[]{}".contains(c);
    let mut out = String::with_capacity(text.len());
    let (mut word, mut follows_key) = (String::new(), false);
    let flush = |word: &mut String, out: &mut String, follows_key: &mut bool| {
        if !word.is_empty() {
            let (masked, next) = mask_word(word, *follows_key);
            out.push_str(&masked);
            *follows_key = next;
            word.clear();
        }
    };
    for c in text.chars() {
        if separator(c) {
            flush(&mut word, &mut out, &mut follows_key);
            if c == '\n' {
                follows_key = false;
            }
            out.push(c);
        } else {
            word.push(c);
        }
    }
    flush(&mut word, &mut out, &mut follows_key);
    out
}

/// Writes the chosen logs (masked tails) to `out/<name>`; unknown or missing logs are skipped.
pub fn write_zip(app: &Path, chosen: &[String], out: &Path, name: &str) -> Result<PathBuf, String> {
    let path = out.join(name);
    let file = fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for source in sources(app) {
        if source.id != ENVIRONMENT && !chosen.iter().any(|c| c == source.id) {
            continue;
        }
        let text = if source.id == ENVIRONMENT {
            environment()
        } else {
            match source.path.as_deref().and_then(read_tail) {
                Some(text) => mask(&text),
                None => continue,
            }
        };
        zip.start_file(source.name, options)
            .map_err(|e| e.to_string())?;
        zip.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(path)
}
