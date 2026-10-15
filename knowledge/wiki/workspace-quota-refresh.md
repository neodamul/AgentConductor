# Workspace quota collection and agent controls

The minimum-only presentation is superseded by [Compact quota windows](compact-quota-windows.md).

## Correction and scope

DD-67 display fixtures validated rendering but missed collection when a workspace had no
last run. The operator's active workspace had no run, so both badges were unavailable.
The fresh-workspace request regression failed before the new metadata command existed.

Workspace metadata jobs now query each registered paid CLI independently and persist
profile-bound observations in workspace `quota.sqlite`. Run views may reuse this cache
only when the workspace explicitly owns the exact run. Newer observations win; profile
changes invalidate old observations. Account capacity is neither money nor context size.

DD-69 schedules each supported agent at startup and every two minutes while the broker
is idle. Manual refresh affects only that agent's deadline. Each name has a refresh
control, with a busy indication for its job. Unsupported providers have disabled quota
controls. Edit has a pencil icon; Models has a reload icon after its label; Remove is a
trash icon with an accessible name. No new icon dependency is required.

## Authenticated evidence and limits

On 2026-10-11, actual Codex `account/rateLimits/read` returned 75% and 96% remaining;
Claude 2.1.287 screen-reader `/usage` returned 97% session and 56% weekly remaining.
These are time-specific account observations, not seeded display fixtures.
The owned Claude metadata probe reported zero API duration and zero total cost and
its process group was quiescent afterward. The probe disables tools and hooks, sends
only `/usage`, never accepts a trust prompt, and has bounded output and a timeout.
The user explicitly approved trust for the canonical Development repository.

The terminal adapter is verified on macOS with this account; Enterprise account behavior
and Windows terminal collection remain unverified. Other platforms retain Claude
stream observations. Human-readable reset text is not converted to invented timestamps.
GitHub Windows runners remain blocked by account billing; local success is not Windows
runtime evidence. Native screenshot/interaction verification is not claimed here.

## Rust-owned refresh cadence (DD-161)

Startup and Agent Room now pass each agent's last accepted refresh time to their native
Rust snapshot request. Rust computes per-profile due state using the configured 1–60
minute interval, with a five-minute fallback for invalid values. Only Claude and Codex
are scheduled. The UI continues to enqueue refresh work asynchronously; accepted startup
refreshes advance the cooldown, rejected requests remain eligible for retry, and changing
the Claude project binding clears its last-refresh timestamp. JavaScript no longer owns a
second interval predicate.

Verification on 2026-10-15: all-target Rust regression (502 passed, 14 ignored, 1 slow
repository source-layout scan filtered), all 44 remaining Python contracts, focused
startup/room/quota tests, `cargo fmt --check`, all-target Clippy with warnings denied,
and `git diff --check` passed. Changed Rust source and integration-test files remain below
500 lines. The filtered source-layout test was not counted as passing.

## Verification

- Full local regression before the last cache-binding case: 272 passed, 12 ignored.
- Final focused quota, startup, native-room, layout and UI checks: 26 passed, 1 ignored.
- Opt-in authenticated Claude collector test: 1 passed, including zero-generation and cleanup.
- Python/Node display and refresh-boundary regression: 1 passed.
- JavaScript syntax, Rust formatting, Clippy with warnings denied and optimized build.

## Runtime replacement

The previous owned native app was terminated and its absence checked before relaunch.
The replacement app independently refreshed the workspace without a last run: Codex
68%/95% and Claude 93%/56% remaining, both without collection errors and observed
within 13 seconds of the cache check. One native app process was present.

## Source fingerprints

- `src/commands/quota.rs` SHA-256 `38e244b51f035dbc9c903dce85fb9ce92472db739eace4e46ad63f6646c320a5`
- `src/quota/terminal.rs` SHA-256 `c2c463a64c94ccccf887c8802e163d3c29e6a5866013e4a9be4fdbd4878f1d80`
- `src/quota/workspace.rs` SHA-256 `3699c1c1f8e33fb7a37370f1280b83dc19d2feb76551b43ffaea14a6300a31eb`
- `src/ui/startup.js` SHA-256 `c52c625998ae171e673e617bbd90c555bcd88e9f2cacd954bc52ba9e647a4ae8`
- `src/ui/room.js` SHA-256 `cd4f40a58820cb0b312e1d51e780f96dc45692fe87efbcf0cfc940cf9db56e92`
- `src/quota/refresh.rs` SHA-256 `cc8b97a765066113474be81a33dfe366fb081a40d580eed09dd678689826cb7d`
- `src/ui/controller.rs` SHA-256 `b2f321da7d63dc7b75d2ca7775a223ee8ff8cde08bea5b45e936ef0c5b204324`
- `src/ui/setup/snapshot.rs` SHA-256 `3566bbf08ffeacb96bbe70a17affbe07e8e9b1b4838b4dfeba7f010cdaa17de0`
- `tests/quota_refresh.rs` SHA-256 `072192226cfe74624babc31178ce6d2b57134e9d6e85447d5463361bc2dce14b`
- `tests/claude_usage.rs` SHA-256 `c00fae7b26b8e467fdd296f7ad8b0e9eac2a288b5d99d1b03be3a74cd753a925`
- `tests/startup_quota.rs` SHA-256 `dae4d89618c55bcda1b379c3c3af439c3e54d134164e1158de2b79cf5f93e998`
- `tests/test_quota_badge.py` SHA-256 `600e0cf9279c8d0bacacd223861ed60d0d9bb93180ee76679bfedf87ca0fd663`
