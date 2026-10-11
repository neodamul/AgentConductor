# MCP goal imports (planned)

R-62 → BG-23 → SF-52 → UC-51/52 → FR-47/48, NFR-22 → QAS-53–55 → DD-68.
This is a deferred requirement; no connector, UI or network access has been implemented.

Workspace settings use OpenCode's `mcp` map and server-object schema, with separate
application bindings selecting which registered server/tool resolves Jira and Confluence.
The same server may serve both. Example configuration (operator-provided endpoints):

```json
{
  "mcp": {
    "company-atlassian": {
      "type": "remote",
      "url": "https://mcp.example.com/mcp",
      "enabled": true,
      "timeout": 5000
    },
    "company-local": {
      "type": "local",
      "command": ["company-mcp", "serve"],
      "environment": {},
      "enabled": false
    }
  }
}
```

The compatibility baseline is the [official OpenCode MCP configuration documentation](https://opencode.ai/docs/mcp-servers/)
checked 2026-10-11. Remote entries support URL, headers and OAuth configuration; local
entries use argv commands, working directory and environment. Connection/authentication
secrets are local and excluded from portable settings; model API keys are not required.
Do not copy tokens from OpenCode's auth storage. Review the selected connector's license
and company deployment terms before installing or bundling it.

Canonical references: `jira:EMM-52599`, `confluence:53553565`, `confluence:548443335`.
The source picker inserts typed chips. Bare Story keys/reference-number lists produce
suggestions requiring confirmation; unrelated numbers never cause document access.
Multiple tenants require explicit binding selection. Preserve the original goal text.

Import previews read-only Story fields/acceptance criteria and selected page contents,
then freezes bounded local source snapshots with IDs, URLs, timestamps, versions and
digests before planning. Missing records, auth errors and partial imports are explicit.
Refresh requires a new preview/review. Imported content cannot alter planning rules or
permission/approval gates; source-linked LLM-wiki summaries do not automatically expose
private document bodies through shared memory or settings exports.

The implementation checklist is in docs/07; registration, references, retrieval and
planning integration are separate TDD increments with fixture and authorized live checks.
