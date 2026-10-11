# Windows CLI metadata parity

Trace: R-72 / DD-78. npm installs extensionless POSIX wrappers beside Windows launchers. Select .exe/.cmd before an extensionless file and resolve retained Codex profile paths at query time. Claude /usage needs an actual terminal: use bounded, on-demand ConPTY with a kill-on-close job; never send project-trust approval or a generation prompt. Runtime output remains local.

Installed desktop artwork is optional. Original CL/CX/OC monograms provide bounded PNG fallback badges without a vendor desktop dependency or redistributed vendor graphics. Legacy GDI icons may have zero alpha; make that legacy buffer opaque before encoding.

MCP configuration belongs in Settings, while reference insertion and preview remain next to the goal. The top-bar gear is 26 px in a 44 × 44 px target.

Verification: Windows executable, Codex batch metadata and Claude terminal fixtures are in tests/windows_metadata.rs. Actual signed-in enterprise accounts require separate operator acceptance.

## Redirected host handles

Windows CI exposed ConPTY child output escaping into the host log. The startup structure now sets `STARTF_USESTDHANDLES` with null handles so Windows binds the child to the pseudoconsole. This follows [Microsoft Terminal maintainer guidance](https://github.com/microsoft/terminal/discussions/15814). The terminal fixture must return both usage windows and prove process cleanup while the test host output is redirected.
