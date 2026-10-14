#!/usr/bin/env python3
"""Offline coordinator fixture: selects MCP tools and reads saved source evidence."""
import json, sys
from pathlib import Path
prompt=json.loads(sys.stdin.read())
mode=sys.argv[1] if len(sys.argv)>1 else "normal"
if prompt["task"] == "summarize":
    assert "jira:AC-1" in prompt["goal"], "summary must receive the actual goal"
    raw=Path(prompt["source_path"]).read_text(encoding="utf-8")
    assert "TAIL_REQUIREMENT" in raw or "Replay is rejected" in raw
    reply={"summary":"Preserve single-use refresh tokens. Replay is rejected. TAIL_REQUIREMENT: retain audit history"}
else:
    ref=prompt["reference"]
    catalog=prompt["catalog"]
    candidates=[t for t in catalog if t.get("name")==("jira_get_issue" if ref["kind"]=="jira" else "confluence_get_page")]
    if not candidates:
        catalog=[{**t,"name":"jira_get_issue" if ref["kind"]=="jira" else "confluence_get_page","actual_name":t["name"]} for t in catalog if t.get("name")=="get_issue"]
    if not candidates and not catalog:
        server=prompt["catalog"][0]["server"]
        print(json.dumps({"type":"result","payload":{"selection":{"action":"inspect","server":server}}}));sys.exit(0)
    selected=next(t for t in catalog if (ref["kind"]!="jira" or t["server"]!="wiki-server") and (ref["kind"]!="confluence" or t["server"]!="jira-server") and t.get("name")==("jira_get_issue" if ref["kind"]=="jira" else "confluence_get_page"))
    tool="deleteIssue" if mode=="write" else selected["name"]
    args={"id" if selected.get("actual_name") else "issue_key" if ref["kind"]=="jira" else "page_id":ref["id"]}
    if mode == "unrelated":
        args={key:"AC-2" for key in args}
    helpers=[t for t in catalog if t.get("name")=="getAccessibleAtlassianResources"]
    if helpers and prompt["observations"]:
        args["cloudId"]=json.loads(prompt["observations"][-1]["result"])["structuredContent"][0]["id"]
    reply={"selection":{"action":"read","server":selected["server"],"tool":selected.get("actual_name",tool) if mode!="write" else tool,"arguments":args}}
    if helpers and not prompt["observations"]:
        reply={"selection":{"action":"helper","server":helpers[0]["server"],"tool":helpers[0]["name"],"arguments":{}}}
print(json.dumps({"type":"result","payload":reply}))
