use super::*;

pub(in crate::mcp) async fn tools(c: &mut Client) -> Result<Vec<Value>, String> {
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
