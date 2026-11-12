# Windows MCP schema reads must use CLI-compatible paths

Source: [fingerprinted diagnosis and checks](../raw/mcp-preview-tags-898.json), #898. Trace: R-134 / UC-124 / QAS-130 / DD-143.

An identifier guessed from another server's tool contract can fail even when the tool name matches. Return invalid arguments with the exact discovered schema to the coordinator before transmitting anything; never hard-code field names from one deployment.

The operator traced the next failure to a full-schema Read: Windows canonicalization introduced an extended drive prefix that Claude treated as UNC. Preserve canonical OS paths for storage, but normalize CLI-facing paths using the existing path helper. Supply the complete schema inline when it fits the bounded prompt; retain a full local artifact and normalized file fallback for larger schemas. Bounded source Markdown can likewise avoid an extra file-tool round trip.

Selection rounds and provider capacity/format retries share one five-minute deadline per reference. Summaries have their own five-minute deadline. Cancel owned processes on expiry and name the stage/reference without dumping a raw provider JSON tail. Do not disable installed startup hooks based only on an unconfirmed delay correlation.

Source references retain canonical lowercase identities while their visible bracketed text stays editable. Highlight only exact UTF-16 source spans; preserve a transparent editor and aligned backing layer.

The MCP contract already provides `name` and `inputSchema` through `tools/list`, and `tools/call` accepts the name plus arguments. The failed request did not reach Jira: repeated cold CLI starts and the redundant schema Read were orchestration overhead, not proof of a slow Jira API. A schema identifies names/types/required fields, while reference-specific values still need resolution. [MCP tool specification](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/docs/specification/2025-06-18/server/tools.mdx).

## Inspect contract compatibility

Source: [fingerprinted #901 regression](../raw/mcp-inspect-contract-901.json). A valid server/tool can be reported as unavailable when response parsing fails: the generic prompt listed arguments for every action, but strict Inspect forbade that field. Accept only absent/null/empty-object arguments for inspection and advertise action-specific fields. Keep unknown-field, nonempty-argument and downstream read-only/schema rejection. Inspection itself must not invoke tools/call; trace the final read separately.
