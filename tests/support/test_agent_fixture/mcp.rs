use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, Write},
    process::Command,
    time::Duration,
};

// This fixture leaves a child alive so the coordinator can verify process-group cleanup.
#[allow(clippy::zombie_processes)]
pub fn server(args: &[String]) {
    let mode = args.first().map(String::as_str).unwrap_or("normal");
    let trace = args.get(1);
    if mode == "hang" {
        let leak = format!("{}.leak", trace.map(String::as_str).unwrap_or("trace"));
        Command::new(std::env::current_exe().unwrap())
            .args(["mcp-leak", leak.as_str()])
            .spawn()
            .unwrap();
        std::thread::sleep(Duration::from_secs(60));
        return;
    }
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        let request: Value = serde_json::from_str(&line).unwrap();
        if let Some(path) = trace {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();
            writeln!(
                file,
                "{}",
                serde_json::to_string(&request)
                    .unwrap()
                    .replace("\":", "\": ")
            )
            .unwrap();
        }
        let Some(id) = request.get("id") else {
            continue;
        };
        if mode == "malformed" {
            println!("not-json");
            std::io::stdout().flush().unwrap();
            continue;
        }
        let method = request["method"].as_str().unwrap_or_default();
        let mut result = match method {
            "initialize" => json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}}}),
            "tools/list" => tools(mode),
            "tools/call" => call(mode, &request),
            other => panic!("unexpected MCP method: {other}"),
        };
        if mode == "large_sources" && method == "tools/call" {
            result["structuredContent"]["description"] = json!(format!(
                "{}TAIL_REQUIREMENT: retain audit history",
                "큰 문서 요구사항\n".repeat(20_000)
            ));
        }
        if mode == "oversized" {
            result["padding"] = json!("x".repeat(9 * 1024 * 1024));
        }
        println!("{}", json!({"jsonrpc":"2.0","id":id,"result":result}));
        std::io::stdout().flush().unwrap();
    }
}

fn tools(mode: &str) -> Value {
    let mut tools = vec![
        json!({"name":"get_issue","inputSchema":{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false},"annotations":{"readOnlyHint":mode!="write","destructiveHint":mode=="write"}}),
    ];
    if matches!(mode, "sources" | "large_sources" | "cloud_sources") {
        tools = vec![
            json!({"name":"jira_get_issue","inputSchema":{"type":"object","properties":{"issue_key":{"type":"string"}},"required":["issue_key"],"additionalProperties":false},"annotations":{"readOnlyHint":true}}),
            json!({"name":"confluence_get_page","inputSchema":{"type":"object","properties":{"page_id":{"type":"string"}},"required":["page_id"],"additionalProperties":false},"annotations":{"readOnlyHint":true}}),
        ];
    }
    if mode == "cloud_sources" {
        for tool in &mut tools {
            tool["inputSchema"]["properties"]["cloudId"] = json!({"type":"string"});
            tool["inputSchema"]["required"]
                .as_array_mut()
                .unwrap()
                .push(json!("cloudId"));
        }
        tools.push(json!({"name":"getAccessibleAtlassianResources","inputSchema":{"type":"object","additionalProperties":false},"annotations":{"readOnlyHint":true}}));
    }
    if mode == "schema" {
        tools[0]["inputSchema"]["required"] = json!(["tenant"]);
    }
    if mode == "many_tools" {
        tools[0]["description"] = json!("read document ".repeat(100));
        for i in 0..127 {
            tools.push(json!({"name":format!("get_dummy_{i}"),"description":"metadata ".repeat(100),"inputSchema":{"type":"object"},"annotations":{"readOnlyHint":true}}));
        }
    }
    let mut result = json!({"tools":tools});
    if mode == "paging" {
        result["nextCursor"] = json!("again");
    }
    result
}

fn call(mode: &str, request: &Value) -> Value {
    let params = &request["params"];
    let args = &params["arguments"];
    let key = args
        .get("id")
        .or_else(|| args.get("issue_key"))
        .or_else(|| args.get("page_id"))
        .cloned()
        .unwrap_or(Value::Null);
    if params["name"] == "getAccessibleAtlassianResources" {
        return json!({"structuredContent":[{"id":"tenant-1","url":"https://example.org"}]});
    }
    if mode == "cloud_sources" && args["cloudId"] != "tenant-1" {
        return json!({"isError":true,"content":[]});
    }
    if key == "999" {
        return json!({"isError":true,"content":[{"type":"text","text":"secret server details must not be logged"}]});
    }
    json!({"structuredContent":{"id":key,"title":"Rotation","description":"Use single-use refresh tokens.","acceptanceCriteria":["Replay is rejected"],"url":format!("https://example.org/browse/{}",key.as_str().unwrap_or_default())}})
}

pub fn leak(args: &[String]) {
    std::thread::sleep(Duration::from_secs(1));
    fs::write(args.first().expect("leak path"), "leaked").unwrap();
}
