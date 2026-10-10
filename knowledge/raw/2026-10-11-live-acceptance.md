# Live acceptance re-run (2026-10-11)

- Claude 2.1.287 (existing claude.ai login, haiku): `native_claude` 2/2, `native_control` 1/1,
  `native_claude_workflow` 1/1 (three guarded workers → approve → merge), new
  `native_claude_denial` 1/1 run twice.
- Denial evidence: the out-of-scope Write was refused by policy and the owned process group
  was terminated before any write ("tool write is outside the task's owned scopes"). The
  human-rejected `touch denied-marker.txt` never ran; that worker continued to completion.
- Codex 0.162.1: both live tests stopped at `turn.failed` "You've hit your usage limit …
  try again at Oct 16th, 2026 9:59 AM". The detector classifies it as a restriction with no
  invented reset time (local date without offset) → manual schedule; regression test added.
- Codex also reported malformed user role files in ~/.codex/agents (developer.toml,
  tester.toml without `name`); non-terminal warnings, user configuration, untouched.
