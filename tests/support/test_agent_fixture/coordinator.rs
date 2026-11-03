use serde_json::{Value, json};
use std::{io::Read, path::Path};

pub fn run(args: &[String]) {
    let mode = args.first().map(String::as_str).unwrap_or("normal");
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let prompt: Value = serde_json::from_str(&input).unwrap();
    let reply = if prompt["task"] == "summarize" {
        assert!(prompt["goal"].as_str().unwrap().contains("jira:AC-1"));
        let source =
            std::fs::read_to_string(Path::new(prompt["source_path"].as_str().unwrap())).unwrap();
        assert!(source.contains("TAIL_REQUIREMENT") || source.contains("Replay is rejected"));
        json!({"summary":"Preserve single-use refresh tokens. Replay is rejected. TAIL_REQUIREMENT: retain audit history"})
    } else {
        select(&prompt, mode)
    };
    println!("{}", json!({"type":"result","payload":reply}));
}

fn select(prompt: &Value, mode: &str) -> Value {
    let reference = &prompt["reference"];
    let wanted = if reference["kind"] == "jira" {
        "jira_get_issue"
    } else {
        "confluence_get_page"
    };
    let catalog = prompt["catalog"].as_array().unwrap();
    let mut candidates: Vec<Value> = catalog
        .iter()
        .filter(|tool| tool["name"] == wanted)
        .cloned()
        .collect();
    if candidates.is_empty() {
        candidates = catalog
            .iter()
            .filter(|tool| tool["name"] == "get_issue")
            .map(|tool| {
                let mut result = tool.clone();
                result["actual_name"] = tool["name"].clone();
                result["name"] = json!(wanted);
                result
            })
            .collect();
    }
    if candidates.is_empty() {
        return json!({"selection":{"action":"inspect","server":catalog.first().map(|t|t["server"].clone()).unwrap_or(Value::Null)}});
    }
    let selected = candidates
        .iter()
        .find(|tool| {
            let server = tool["server"].as_str().unwrap_or_default();
            (reference["kind"] != "jira" || server != "wiki-server")
                && (reference["kind"] != "confluence" || server != "jira-server")
        })
        .unwrap_or_else(|| candidates.first().unwrap());
    let tool = if mode == "write" {
        "deleteIssue"
    } else {
        selected["actual_name"].as_str().unwrap_or(wanted)
    };
    let id_key = if selected.get("actual_name").is_some() {
        "id"
    } else if reference["kind"] == "jira" {
        "issue_key"
    } else {
        "page_id"
    };
    let id = if mode == "unrelated" {
        json!("AC-2")
    } else {
        reference["id"].clone()
    };
    let mut arguments = json!({id_key:id});
    let helpers: Vec<&Value> = catalog
        .iter()
        .filter(|entry| entry["name"] == "getAccessibleAtlassianResources")
        .collect();
    let observations = prompt["observations"].as_array();
    if !helpers.is_empty() && observations.is_some_and(|items| !items.is_empty()) {
        let result = &observations.unwrap().last().unwrap()["result"];
        let parsed = result
            .as_str()
            .and_then(|value| serde_json::from_str::<Value>(value).ok())
            .unwrap_or_else(|| result.clone());
        arguments["cloudId"] = parsed["structuredContent"][0]["id"].clone();
    }
    if !helpers.is_empty() && observations.is_none_or(Vec::is_empty) {
        return json!({"selection":{"action":"helper","server":helpers[0]["server"],"tool":helpers[0]["name"],"arguments":{}}});
    }
    json!({"selection":{"action":"read","server":selected["server"],"tool":tool,"arguments":arguments}})
}
