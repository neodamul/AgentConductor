#!/usr/bin/env python3
"""Offline MCP protocol fixture; never calls a model or company service."""
import json, os, sys, time, subprocess
mode = sys.argv[1] if len(sys.argv) > 1 else "normal"
trace = sys.argv[2] if len(sys.argv) > 2 else None
if mode == "hang":
    subprocess.Popen([sys.executable, "-c", "import time,pathlib,sys;time.sleep(1);pathlib.Path(sys.argv[1]).write_text(\"leaked\")", trace + ".leak"])
    time.sleep(60)
for line in sys.stdin:
    request = json.loads(line)
    if trace:
        with open(trace, "a", encoding="utf-8") as f:
            f.write(json.dumps(request) + "\n")
    if "id" not in request:
        continue
    method = request["method"]
    if mode == "malformed":
        print("not-json", flush=True)
        continue
    if method == "initialize":
        result = {"protocolVersion": "2025-06-18", "capabilities": {"tools": {}}}
    elif method == "tools/list":
        result = {"tools": [{"name": "get_issue", "inputSchema": {"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"], "additionalProperties": False}, "annotations": {"readOnlyHint": mode != "write", "destructiveHint": mode == "write"}}]}
        if mode == "paging":
            result["nextCursor"] = "again"
        if mode == "schema":
            result["tools"][0]["inputSchema"]["required"] = ["tenant"]
        if mode in ("sources", "large_sources", "cloud_sources"):
            result["tools"] = [
                {"name": "jira_get_issue", "inputSchema": {"type": "object", "properties": {"issue_key": {"type": "string"}}, "required": ["issue_key"], "additionalProperties": False}, "annotations": {"readOnlyHint": True}},
                {"name": "confluence_get_page", "inputSchema": {"type": "object", "properties": {"page_id": {"type": "string"}}, "required": ["page_id"], "additionalProperties": False}, "annotations": {"readOnlyHint": True}},
            ]
        if mode == "cloud_sources":
            for tool in result["tools"]:
                tool["inputSchema"]["properties"]["cloudId"] = {"type": "string"}
                tool["inputSchema"]["required"].append("cloudId")
            result["tools"].append({"name":"getAccessibleAtlassianResources", "inputSchema":{"type":"object","additionalProperties":False}, "annotations":{"readOnlyHint":True}})
        if mode == "many_tools":
            result["tools"][0]["description"] = "read document " * 100
            result["tools"].extend({"name":"get_dummy_"+str(i),"description":"metadata "*100,"inputSchema":{"type":"object"},"annotations":{"readOnlyHint":True}} for i in range(127))
    elif method == "tools/call":
        args = request["params"]["arguments"]
        key = args.get("id") or args.get("issue_key") or args.get("page_id")
        if request["params"]["name"] == "getAccessibleAtlassianResources":
            result = {"structuredContent":[{"id":"tenant-1","url":"https://example.org"}]}
        elif mode == "cloud_sources" and args.get("cloudId") != "tenant-1":
            result = {"isError":True,"content":[]}
        elif key == "999":
            result = {"isError": True, "content": [{"type": "text", "text": "secret server details must not be logged"}]}
        else:
            result = {"structuredContent": {"id": key, "title": "Rotation", "description": "Use single-use refresh tokens.", "acceptanceCriteria": ["Replay is rejected"], "url": "https://example.org/browse/" + key}}
    else:
        raise RuntimeError("unexpected method")
    if mode == "large_sources" and method == "tools/call":
        result["structuredContent"]["description"] = "큰 문서 요구사항\n" * 20000 + "TAIL_REQUIREMENT: retain audit history"
    if mode == "oversized":
        result["padding"] = "x" * (9 * 1024 * 1024)
    print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "result": result}), flush=True)
