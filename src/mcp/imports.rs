mod references;
mod store;

use super::{Settings, transport::Client};
pub use references::{Reference, references};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;
pub use store::{apply, context, load, store};
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
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub summaries: std::collections::BTreeMap<String, super::sources::Summary>,
}
impl Preview {
    pub(super) fn digest(&self) -> Result<String, String> {
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
pub(super) async fn tools(c: &mut Client) -> Result<Vec<Value>, String> {
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
        summaries: super::sources::empty(),
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
                Ok(record) => p.records.push(record),
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
pub(super) async fn read_one(
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
    if body.len() > super::MAX_SOURCE {
        return Err(crate::ui::messages::text("mcp.source_too_large").into());
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
