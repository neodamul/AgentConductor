#!/usr/bin/env python3
"""Offline coordinator fixture: selects MCP tools and reads saved source evidence."""
import json, sys
from pathlib import Path
prompt=json.loads(sys.stdin.read())
mode=sys.argv[1] if len(sys.argv)>1 else "normal"
if prompt["task"] == "summarize":
    assert not prompt["source_path"].startswith(chr(92)*2+"?"), "CLI must receive an ordinary Windows path"
    assert "jira:ac-1" in prompt["goal"].lower(), "summary must receive the actual goal"
    if mode == "inline":
        raw=prompt["source_text"]
        assert isinstance(raw,str) and "# Full source" in raw
    else:
        raw=Path(prompt["source_path"]).read_text(encoding="utf-8")
    assert "TAIL_REQUIREMENT" in raw or "Replay is rejected" in raw
    reply={"summary":"Preserve single-use refresh tokens. Replay is rejected. TAIL_REQUIREMENT: retain audit history"}
else:
    if mode in ("schema_inline", "schema_file"):
        observations=prompt["observations"]
        assert all(not o.get("schema_path","").startswith(chr(92)*2+"?") for o in observations)
        if not observations:
            print(json.dumps({"type":"result","payload":{"selection":{"action":"inspect","server":"docs","tool":"jira_get_issue"}}}));sys.exit(0)
        if mode == "schema_file":
            schema_path=next(o["schema_path"] for o in observations if "schema_path" in o)
            assert not schema_path.startswith(chr(92)*2+"?")
            schema=json.loads(Path(schema_path).read_text(encoding="utf-8"))
            assert schema["inputSchema"]["description"].endswith("SCHEMA_TAIL_REQUIRED")
        else:
            schema=next(o["schema"] for o in observations if "schema" in o)
        assert schema["name"] == "jira_get_issue"
        assert "issue_key" in schema["inputSchema"]["properties"]
    if mode == "strict_issue":
        observations=prompt["observations"]
        args={"issueIdOrKey":prompt["reference"]["id"]}
        if observations:
            schema=observations[-1]["schema"]["inputSchema"]
            assert observations[-1]["validation_error"]
            assert set(schema["required"])=={"issueKey","fieldIds","expand"}
            args={"issueKey":prompt["reference"]["id"],"fieldIds":["summary","description"],"expand":"renderedFields"}
        print(json.dumps({"type":"result","payload":{"selection":{"action":"read","server":"docs","tool":"getIssue","arguments":args}}}));sys.exit(0)
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
