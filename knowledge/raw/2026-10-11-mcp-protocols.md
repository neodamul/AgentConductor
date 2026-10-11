# MCP protocol and implementation sources — 2026-10-11

Trace: R-62 → BG-23 → SF-52 → UC-51–52 → FR-47–48/NFR-22 → QAS-53–55 → DD-68.

Primary references checked:

- https://opencode.ai/docs/mcp-servers/ — mcp server map; local command/cwd/environment/enabled/timeout, remote URL/headers/oauth/enabled/timeout.
- https://modelcontextprotocol.io/specification/2025-06-18/basic/transports — stdin/stdout JSONL and Streamable HTTP JSON/SSE; initialization, negotiated protocol header and session termination.

The built-in client is original MIT project code, uses existing Tokio/process containment,
and adds no Cargo dependency or distributed MCP server. Remote transport uses the host
curl executable. Local Python servers are offline test fixtures, not runtime dependencies.
No company document bodies or credentials are included in this knowledge asset.

TDD evidence: missing mcp module first failed configuration tests; missing native controls
first failed UI tests; SSE fixture holding its connection open reproduced a timeout before
incremental response parsing; three same-server imports first launched three sessions before
per-server session reuse made the one-session assertion pass. Workspace worker tests cover
preview/apply, goal invalidation and portable export/import without private bodies. Source
hash changes and malicious write annotations are refused. See local regression records for
final counts. Native OAuth authorization and legacy HTTP+SSE are explicit limitations, with
an authenticated local proxy supported as the company authentication path.
