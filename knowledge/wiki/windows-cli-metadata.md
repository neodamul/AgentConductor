# Windows CLI metadata parity

Trace: R-72 / DD-78. npm installs extensionless POSIX wrappers beside Windows launchers. Select .exe/.cmd before an extensionless file and resolve retained Codex profile paths at query time. Claude /usage needs an actual terminal: use bounded, on-demand ConPTY with a kill-on-close job; never send project-trust approval or a generation prompt. Runtime output remains local.

Installed desktop artwork is optional. Original CL/CX/OC monograms provide bounded PNG fallback badges without a vendor desktop dependency or redistributed vendor graphics. Legacy GDI icons may have zero alpha; make that legacy buffer opaque before encoding.

MCP configuration belongs in Settings, while reference insertion and preview remain next to the goal. The top-bar gear is 26 px in a 32 × 32 px target.

Verification: Windows executable, Codex batch metadata and Claude terminal fixtures are in tests/windows_metadata.rs. Actual signed-in enterprise accounts require separate operator acceptance.

## Redirected host handles

Windows CI exposed ConPTY child output escaping into the host log. The startup structure now sets `STARTF_USESTDHANDLES` with null handles so Windows binds the child to the pseudoconsole. This follows [Microsoft Terminal maintainer guidance](https://github.com/microsoft/terminal/discussions/15814). The terminal fixture must return both usage windows and prove process cleanup while the test host output is redirected.

## Batch launcher command line

A forward slash in the cmd.exe executable path was interpreted as a command switch in Windows CI. Build the system executable path with separate native path components, use the literal `cmd.exe` argv[0], and quote both the batch file and safety settings file. The fixture includes a spaced executable path and verifies native and batch terminal quota collection, trust refusal and cleanup.

Cross-platform lint checks also matter: Windows ConPTY makes local MCP state larger than on macOS, so the transport boxes that variant. Use borrowed slices in Windows-only test code; macOS cannot lint code behind a Windows cfg. Both functional suites passed before these platform lint corrections; final packaging requires every check to pass.

## Native verification and stable delivery

[CI 38134378978](https://github.com/neodamul/AgentConductor/actions/runs/38134378978) passed every validation and native package job on 5cefa6c. Windows: 135 Rust tests, 0 failures, 1 ignored; 32 Python contracts; fmt and Clippy. Both released binaries also passed parallel shared/worktree execution and approval/merge smoke checks. Stable [v0.1.2](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.2) includes source manifests, archive/binary checksums, license texts and SBOM. Real operator subscription acceptance remains distinct from deterministic fixtures.
