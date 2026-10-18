# Windows MCP auto-source test isolation

## Evidence

Windows Actions run `39143064496` failed all four `mcp_auto_sources` cases while the macOS target passed. The Windows log shows an MCP timeout, a missing trace file, a coordinator `StopIteration`, and a missing compact-catalog assertion. The same MCP tests passed on the current main-based branch on macOS.

The failures are consistent with the four async cases starting several Python MCP child processes concurrently while relying on the 5-second default request deadline. When catalog discovery does not finish, downstream selection assertions observe incomplete catalogs; the denied-tool test also previously failed with an unhelpful `NotFound` when it read a trace that was never created.

## Change and verification

The fixture now allows 30 seconds for local MCP requests and requires `tools/list` evidence before checking that a denied selection never calls a tool. This keeps the authorization assertion strict and distinguishes fixture startup failure from a successful denial. Focused macOS tests pass 4/4; source-layout, Clippy, formatting, and diff checks pass. Follow-up Actions run `39146502900` passed the full all-target suite on Windows and macOS, including source layout, native UI launch, formatting, and lint validation.

## Limits

This is test-fixture hardening; production MCP request defaults are unchanged. The Actions workflow's separate native-package jobs were still running when this evidence was recorded; package validation is outside the #33 acceptance criteria.
