# MCP goal sources

AgentConductor treats external documents as human-reviewed evidence for a goal, never as
execution authority. OpenCode-shaped connection settings and exact read-tool bindings are
separate from agent CLI authentication. On-demand discovery/import uses contained local
JSONL processes or short-lived curl Streamable HTTP. Reuse one session per server and request
large UI results once to respect the lightweight-host requirement.

Preview/apply stores goal/settings/content digests and immutable local snapshots. Required
Jira stories cannot be skipped after failed reads; optional Confluence pages can be excluded
explicitly. Planning labels selected context untrusted; source modifications require a new
review. Portable settings exclude source bodies/receipts; shared memory receives only the
implementation decisions, never company document content.

Goal-reference extraction and explicit-reference text ranges run in Rust through the existing
native request channel. The UI draws tags from those ranges and renders bounded suggestions;
no document lookup runs until the user previews them.

[Primary sources and TDD evidence](../raw/2026-10-11-mcp-protocols.md).
[Operator guide](../../docs/mcp-sources.md). Trace: R-62/DD-68/QAS-53–55.
