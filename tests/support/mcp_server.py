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
    elif method == "tools/call":
        key = request["params"]["arguments"]["id"]
        if key == "999":
            result = {"isError": True, "content": [{"type": "text", "text": "secret server details must not be logged"}]}
        else:
            result = {"structuredContent": {"id": key, "title": "Rotation", "description": "Use single-use refresh tokens.", "acceptanceCriteria": ["Replay is rejected"], "url": "https://example.org/browse/" + key}}
    else:
        raise RuntimeError("unexpected method")
    if mode == "oversized":
        result["padding"] = "x" * (300 * 1024)
    print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "result": result}), flush=True)
