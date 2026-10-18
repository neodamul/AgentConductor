# Windows MCP auto-source test isolation

## Evidence

Windows Actions run `39143064496` failed all four `mcp_auto_sources` cases while the macOS target passed. The Windows log shows an MCP timeout, a missing trace file, a coordinator `StopIteration`, and a missing compact-catalog assertion. The same MCP tests passed on the current main-based branch on macOS.

The failures are consistent with the four async cases starting several Python MCP child processes concurrently while relying on the 5-second default request deadline. When catalog discovery does not finish, downstream selection assertions observe incomplete catalogs; the denied-tool test also previously failed with an unhelpful `NotFound` when it read a trace that was never created.

## Change and verification

The fixture now allows 30 seconds for local MCP requests and requires `tools/list` evidence before checking that a denied selection never calls a tool. This keeps the authorization assertion strict and distinguishes fixture startup failure from a successful denial. Focused macOS tests pass 4/4; source-layout, Clippy, formatting, and diff checks pass. Windows CI is still required to verify the timing fix on the affected runner.

## Limits

This is test-fixture hardening; production MCP request defaults are unchanged. If Windows CI still fails, inspect process logs for the absent server before changing timeouts or catalog expectations.
