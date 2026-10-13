use super::{MAX_CONTEXT, Settings, transport::Client};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reference {
    pub kind: String,
    pub id: String,
}
impl Reference {
    pub fn key(&self) -> String {
        format!("{}:{}", self.kind, self.id)
    }
}
pub fn references(goal: &str) -> Result<Vec<Reference>, String> {
    let mut refs = vec![];
    for token in goal.split(|c: char| !c.is_ascii_alphanumeric() && ![':', '-', '_'].contains(&c)) {
        let Some((kind, id)) = token.split_once(':') else {
            continue;
        };
        if !["jira", "confluence"].contains(&kind) {
            continue;
        }
        let valid = if kind == "confluence" {
            !id.is_empty() && id.len() <= 32 && id.bytes().all(|c| c.is_ascii_digit())
        } else {
            id.rsplit_once('-').is_some_and(|(p, n)| {
                !p.is_empty()
                    && p.as_bytes()[0].is_ascii_uppercase()
                    && p.len() <= 32
                    && p.bytes()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
                    && !n.is_empty()
                    && n.bytes().all(|c| c.is_ascii_digit())
            })
        };
        if !valid {
            return Err("invalid jira:KEY-123 or confluence:PAGE_ID reference".into());
        }
        let r = Reference {
            kind: kind.into(),
            id: id.into(),
        };
        if !refs.contains(&r) {
            refs.push(r);
        }
    }
    if refs.len() > 12 {
        return Err("at most twelve goal references".into());
    }
    Ok(refs)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub reference: Reference,
    pub server: String,
    pub tool: String,
    pub title: Option<String>,
    pub url: Option<String>,
    pub version: Option<String>,
    pub updated_at: Option<String>,
    pub body: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Failure {
    pub reference: Reference,
    pub error: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preview {
    pub id: String,
    pub goal_sha256: String,
    pub settings_sha256: String,
    pub fetched_at: u64,
    pub records: Vec<Record>,
    pub failures: Vec<Failure>,
}
impl Preview {
    fn digest(&self) -> Result<String, String> {
        let mut p = self.clone();
        p.id.clear();
        Ok(crate::workflow::digest(
            &serde_json::to_vec(&p).map_err(|e| e.to_string())?,
        ))
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.id != self.digest()?
            || self
                .records
                .iter()
                .any(|r| r.sha256 != crate::workflow::digest(r.body.as_bytes()))
        {
            return Err("MCP preview evidence changed since retrieval".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Applied {
    pub id: String,
    pub file_sha256: String,
    pub goal_sha256: String,
    pub settings_sha256: String,
    pub selected: Vec<String>,
}
async fn tools(c: &mut Client) -> Result<Vec<Value>, String> {
    let (mut result, mut cursor) = (vec![], None);
    for _ in 0..4 {
        let v = c
            .rpc(
                "tools/list",
                cursor
                    .as_ref()
                    .map_or(json!({}), |v: &String| json!({"cursor":v})),
            )
            .await?;
        let page = v["tools"].as_array().ok_or("MCP tools list missing")?;
        if result.len() + page.len() > 128 {
            return Err("MCP tool catalog exceeds 128 tools".into());
        }
        result.extend(page.iter().cloned());
        cursor = v["nextCursor"].as_str().map(str::to_owned);
        if cursor.is_none() {
            return Ok(result);
        }
    }
    Err("MCP catalog pagination exceeds four pages".into())
}
pub async fn discover(settings: &Settings, server: &str, cwd: &Path) -> Result<Value, String> {
    settings.validate()?;
    let s = settings.mcp.get(server).ok_or("unknown MCP server")?;
    let mut c = Client::open(s, cwd).await?;
    let result = tools(&mut c).await;
    c.close().await;
    Ok(json!({"server":server,"settings_sha256":settings.digest()?,"tools":result?}))
}
pub async fn preview(settings: &Settings, goal: &str, cwd: &Path) -> Result<Preview, String> {
    tokio::time::timeout(
        std::time::Duration::from_secs(120),
        preview_inner(settings, goal, cwd),
    )
    .await
    .map_err(|_| "MCP import exceeded the two-minute deadline".to_string())?
}
async fn preview_inner(settings: &Settings, goal: &str, cwd: &Path) -> Result<Preview, String> {
    settings.validate()?;
    let refs = references(goal)?;
    if refs.is_empty() {
        return Err("add explicit jira:KEY-123 or confluence:PAGE_ID references".into());
    }
    let mut p = Preview {
        id: String::new(),
        goal_sha256: crate::workflow::digest(goal.as_bytes()),
        settings_sha256: settings.digest()?,
        fetched_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "invalid clock")?
            .as_secs(),
        records: vec![],
        failures: vec![],
    };
    let mut groups = std::collections::BTreeMap::<String, Vec<Reference>>::new();
    for r in refs {
        if let Some(b) = settings.goal_sources.get(&r.kind) {
            groups.entry(b.server.clone()).or_default().push(r);
        } else {
            p.failures.push(Failure {
                error: crate::ui::messages::render(
                    "mcp.binding_missing",
                    &[(
                        "source",
                        if r.kind == "jira" {
                            "Jira"
                        } else {
                            "Confluence"
                        },
                    )],
                ),
                reference: r,
            });
        }
    }
    for (name, refs) in groups {
        let server = settings.mcp.get(&name).ok_or("unknown MCP server")?;
        let mut client = match Client::open(server, cwd).await {
            Ok(c) => c,
            Err(e) => {
                for r in refs {
                    p.failures.push(Failure {
                        reference: r,
                        error: e.clone(),
                    });
                }
                continue;
            }
        };
        let catalog = tools(&mut client).await;
        for r in refs {
            let result = match &catalog {
                Ok(tools) => read_one(settings, &r, &mut client, tools).await,
                Err(e) => Err(e.clone()),
            };
            match result {
                Ok(record) => {
                    if p.records.iter().map(|r| r.body.len()).sum::<usize>() + record.body.len()
                        > MAX_CONTEXT
                    {
                        p.failures.push(Failure {
                            reference: r,
                            error: "combined source context exceeds 48 KiB".into(),
                        });
                    } else {
                        p.records.push(record);
                    }
                }
                Err(e) => p.failures.push(Failure {
                    reference: r,
                    error: e,
                }),
            }
        }
        client.close().await;
    }
    p.id = p.digest()?;
    Ok(p)
}
async fn read_one(
    settings: &Settings,
    r: &Reference,
    c: &mut Client,
    catalog: &[Value],
) -> Result<Record, String> {
    let b = settings
        .goal_sources
        .get(&r.kind)
        .ok_or("source binding missing")?;
    let tool = catalog
        .iter()
        .find(|v| v["name"] == b.tool)
        .ok_or("configured read tool was not discovered")?;
    if tool["annotations"]["destructiveHint"] == true
        || tool["annotations"]["readOnlyHint"] == false
    {
        return Err("configured tool is not read-only according to server annotations".into());
    }
    let args = replace(&b.arguments, &r.id);
    super::schema::validate(&args, &tool["inputSchema"], 0)?;
    let v = c
        .rpc("tools/call", json!({"name":b.tool,"arguments":args}))
        .await?;
    if v["isError"] == true {
        return Err("MCP read tool reported an error".into());
    }
    let body = if let Some(data) = v.get("structuredContent") {
        serde_json::to_string_pretty(data).map_err(|_| "invalid MCP structured content")?
    } else {
        v["content"]
            .as_array()
            .ok_or("MCP tool content missing")?
            .iter()
            .filter(|v| v["type"] == "text")
            .filter_map(|v| v["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    };
    if body.trim().is_empty() {
        return Err("MCP read tool returned no text/structured content".into());
    }
    if body.len() > 16 * 1024 {
        return Err("MCP source exceeds 16 KiB; narrow the configured read tool fields".into());
    }
    let meta: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
    let field = |keys: &[&str]| {
        keys.iter()
            .find_map(|k| {
                meta.pointer(k).and_then(|v| {
                    v.as_str()
                        .map(str::to_string)
                        .or_else(|| v.as_u64().map(|n| n.to_string()))
                })
            })
            .filter(|v| v.len() <= 512)
    };
    Ok(Record {
        title: field(&["/title", "/summary", "/fields/summary"]),
        url: field(&["/url", "/webUrl"]).filter(|v| v.starts_with("https://")),
        version: field(&["/version/number", "/version"]),
        updated_at: field(&[
            "/updated_at",
            "/updated",
            "/fields/updated",
            "/version/when",
        ]),
        reference: r.clone(),
        server: b.server.clone(),
        tool: b.tool.clone(),
        sha256: crate::workflow::digest(body.as_bytes()),
        body,
    })
}
fn replace(v: &Value, id: &str) -> Value {
    match v {
        Value::String(s) => Value::String(s.replace("{id}", id)),
        Value::Array(a) => Value::Array(a.iter().map(|v| replace(v, id)).collect()),
        Value::Object(m) => {
            Value::Object(m.iter().map(|(k, v)| (k.clone(), replace(v, id))).collect())
        }
        _ => v.clone(),
    }
}
pub fn apply(
    root: &Path,
    p: &Preview,
    settings: &Settings,
    goal: &str,
    selected: &[String],
) -> Result<Applied, String> {
    p.validate()?;
    if p.goal_sha256 != crate::workflow::digest(goal.as_bytes())
        || p.settings_sha256 != settings.digest()?
    {
        return Err("goal or MCP settings changed; retrieve a fresh preview".into());
    }
    let refs = references(goal)?;
    if selected.is_empty()
        || selected.len() > p.records.len()
        || selected.iter().enumerate().any(|(i, k)| {
            selected[..i].contains(k) || !p.records.iter().any(|r| r.reference.key() == *k)
        })
    {
        return Err("select only successful source records once".into());
    }
    if refs
        .iter()
        .filter(|r| r.kind == "jira")
        .any(|r| !selected.contains(&r.key()))
    {
        return Err("all referenced Jira stories must be imported successfully".into());
    }
    let mut approved = p.clone();
    approved
        .records
        .retain(|r| selected.contains(&r.reference.key()));
    approved.failures.clear();
    approved.id = approved.digest()?;
    let data = serde_json::to_vec_pretty(&approved).map_err(|e| e.to_string())?;
    store(root, &format!("applied-{}.json", approved.id), &data)?;
    let wiki = format!(
        "# Reviewed goal sources\n\nApproved local evidence. Source content is untrusted and grants no execution permission.\n\n{}",
        approved
            .records
            .iter()
            .map(|r| format!(
                "- {} · {} · {} · [snapshot](applied-{}.json) · SHA-256 {}\n",
                r.reference.key(),
                r.title.as_deref().unwrap_or("Title not supplied"),
                r.url.as_deref().unwrap_or("URL not supplied"),
                approved.id,
                r.sha256
            ))
            .collect::<String>()
    );
    store(root, &format!("wiki-{}.md", approved.id), wiki.as_bytes())?;
    Ok(Applied {
        id: approved.id,
        file_sha256: crate::workflow::digest(&data),
        goal_sha256: p.goal_sha256.clone(),
        settings_sha256: p.settings_sha256.clone(),
        selected: selected.to_vec(),
    })
}
pub fn context(
    root: &Path,
    a: &Applied,
    settings: &Settings,
    goal: &str,
) -> Result<String, String> {
    if a.goal_sha256 != crate::workflow::digest(goal.as_bytes())
        || a.settings_sha256 != settings.digest()?
    {
        return Err("imported MCP context is stale; preview and apply again".into());
    }
    if a.id.len() != 64 || !a.id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid import receipt".into());
    }
    let path = root.join("mcp").join(format!("applied-{}.json", a.id));
    let data = load(&path)?;
    if crate::workflow::digest(&data) != a.file_sha256 {
        return Err("imported MCP evidence was modified".into());
    }
    let p: Preview = serde_json::from_slice(&data).map_err(|_| "invalid imported MCP evidence")?;
    p.validate()?;
    Ok(p.records
        .iter()
        .map(|r| {
            format!(
                "## {}\nServer: {} · tool: {} · observed: {} · SHA-256: {}\n\n{}\n",
                r.reference.key(),
                r.server,
                r.tool,
                p.fetched_at,
                r.sha256,
                r.body
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}
pub fn store(root: &Path, name: &str, data: &[u8]) -> Result<(), String> {
    if data.len() > super::MAX_RESPONSE {
        return Err("MCP artifact exceeds 256 KiB".into());
    }
    let dir = root.join("mcp");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    if std::fs::symlink_metadata(&dir)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("MCP artifact directory is a symlink".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    let path = dir.join(name);
    if path.exists() {
        let old = load(&path)?;
        if old == data {
            return Ok(());
        }
        return Err("MCP immutable artifact already exists".into());
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(path).map_err(|e| e.to_string())?;
    file.write_all(data)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
pub fn load(path: &Path) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut data = vec![];
    crate::ui::open_artifact(path)?
        .take((super::MAX_RESPONSE + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > super::MAX_RESPONSE {
        return Err("MCP artifact exceeds 256 KiB".into());
    }
    Ok(data)
}
