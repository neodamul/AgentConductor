# Jira and Confluence goal sources

Configure company document tools in **Settings → MCP servers…**. The
connection map follows [OpenCode MCP configuration](https://opencode.ai/docs/mcp-servers/).
AgentConductor reads MCP directly; OpenCode installation and model API keys are not
required. Local document bodies stay in the workspace.

Preferred Goal references are `[Jira:EMM-51214]` and `[Confluence:2571110463]`. They stay plain editable text with subdued blue highlights; existing lowercase references remain compatible. The coordinator uses the exact discovered schema, including all required fields; AgentConductor does not assume an identifier field name such as `issueIdOrKey`. Invalid arguments are rejected before MCP transmission and returned with schema feedback for bounded correction.

Selection and retries share a five-minute deadline per reference; each document summary has a separate five-minute deadline. Small complete schemas and source Markdown are passed in stdin to avoid file-tool round trips. Larger evidence remains file-backed; Windows extended drive prefixes are removed only at the CLI-facing boundary. Read-only checks, hashes and human review still apply. A reported slow/cancelled startup hook is not automatically disabled.

## Register a local server

Use **Add server** or **Edit** for the guided form; saving registers the server and discovers its tools. For OpenCode JSON/JSONC, open **Advanced JSON import/export** and import the definitions; importing JSON alone does not start a server.
Replace the example command with your installed, licensed company MCP command.
Use environment references for credentials. AgentConductor does not install a server.

```json
{
  "mcp": {
    "company": {
      "type": "local",
      "command": ["company-mcp", "--stdio"],
      "environment": {"COMPANY_TOKEN": "{env:COMPANY_TOKEN}"},
      "enabled": true,
      "timeout": 5000
    }
  }
}
```

Each saved server row has **Edit**, **Rediscover** and **Remove** actions. Discovery belongs to that server and its result survives reopening or restart. Editing one server preserves the others; changing its definition invalidates that server's previous catalog. Discovery lists tools; document reads occur only when you request a source preview.

Register an agent CLI and choose it as the workspace planning coordinator. On **Preview imports**, it discovers enabled server catalogs, chooses an exact read-only tool and schema-valid arguments for each reference, and can query read-only metadata helpers for required cloud/tenant IDs. AgentConductor validates server, tool, read annotations, schema and reference ID before calling. Writes and undiscovered choices are refused. Missing coordinators show registration guidance before a worker starts.

Existing advanced `goal_sources` definitions remain parseable/exportable for compatibility. The normal UI and worker use coordinator selection and need no manually bound document tools. Standard Codex/Claude profiles use verified read-only coordinator invocations; custom coordinators need an operator-verified read-only configuration.

A local `command` is an argv array, without shell interpolation. Optional `cwd` resolves
from the workspace. Keep installed server commands and their permissions under your
company policy. Disabled servers can remain registered and cannot be used for imports.

## Register a remote server

```json
{
  "mcp": {
    "company": {
      "type": "remote",
      "url": "https://mcp.company.example/mcp",
      "headers": {"Authorization": "Bearer {env:COMPANY_MCP_TOKEN}"},
      "oauth": false,
      "enabled": true,
      "timeout": 5000
    }
  },
  "goal_sources": {}
}
```

Set the named environment variable in the environment inherited by the app. Connector
credentials are separate from model subscriptions. Resolved values are supplied to the
transport privately, never stored in settings or passed as curl arguments. Environment
values and headers must be references (headers also accept `Bearer`/`Basic` prefixes).
The remote client requires host `curl` and uses its normal TLS verification. HTTPS is
required except localhost/127.0.0.1 test servers. URL credentials, query strings and
redirects are refused.

Remote transport supports Streamable HTTP JSON and SSE responses, with session IDs and
session close. Legacy HTTP+SSE endpoints are unsupported. Native OAuth browser login,
dynamic client registration and token refresh are not implemented. OAuth configuration
objects can be retained with environment-referenced credentials; use an authenticated
local MCP proxy or environment-backed headers when authentication is required. No
OpenCode token file is read. An authentication failure does not trigger an implicit
browser or retry loop.

## Import a goal

1. Use **Jira / Confluence** (choose the source in the dialog), or type explicit references:
   `Implement jira:EMM-52599. References confluence:53553565, confluence:548443335.`
2. Unqualified issue keys and a `References (...)` / `참고 (...)` list offer **Use …**
   suggestions. Confirm the ID in the reference dialog; arbitrary prose numbers are
   not imported. Adding a reference does not contact a server.
3. Click **Preview imports**. Workspace settings are saved and one bounded background
   operation discovers tools and uses the registered coordinator to retrieve your IDs.
4. Large documents are first saved in full as immutable local Markdown, then summarized by the coordinator. Review summary, full source snapshot, Markdown path, server/tool, available URL/title/version/update metadata,
   retrieval time and SHA-256. Missing upstream metadata stays unavailable. Jira issues
   are required. Deselect optional Confluence records if appropriate; failed reads are
   shown explicitly and failed Jira reads block Apply.
5. Click **Apply selected sources**, then **Generate plan**. Applying does not run a
   model or execute a task. Generating a plan passes only the selected evidence to your
   chosen planning CLI as untrusted context. Inspect the plan before approving execution;
   final merge still requires its own approval.

Limits are eight servers, twelve references and 128 tools across four catalog pages per server. Retrieval is independent of planning limits: 8 MiB per MCP response, 4 MiB per document, 64 MiB per stored artifact/session output. Summaries are limited to 3 KiB per document and final planning context to 48 KiB; full sources are retained without silent truncation. Per-server requests use the configured 100–60000 ms timeout; coordinator selection/summary calls use 90/180-second deadlines. Attachments and linked documents are not followed automatically.

Discovery/import are on demand: no MCP daemon or background document polling remains.
One local session is reused per server; owned process trees are closed and reaped.

## Local evidence and sharing

The workspace's private `mcp/` directory retains preview JSON, immutable approved
snapshots, full `source-<Markdown SHA-256>.md` files, coordinator summaries and a source-linked wiki index. Source, Markdown and summary hashes are bound to the preview and checked again on apply/planning. Planning keeps `source-context.md` in its local plan
artifacts. Goal/configuration changes invalidate the approval; changed evidence is
refused before execution. Refresh is explicit: preview and apply again, then review a
new plan.

Settings export includes `mcp` and `goal_sources`, with environment references. It
excludes original document bodies and import receipts. Generated plans/slug archives may
contain facts derived from your sources; review them before sharing. Audit events record
source IDs, tool names, statuses and digests without raw document bodies or credentials.
Nothing automatically publishes company documents into shared agentmemory.

No third-party MCP server ships with AgentConductor. The built-in client adds no Rust
crate dependency. Review your selected server's license and company deployment terms
before installing or distributing it.
