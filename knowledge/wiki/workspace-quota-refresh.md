# Workspace quota collection and agent controls

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
- `src/ui/startup.js` SHA-256 `d3beca4d0aa59f9f7dd2cc708dd4541c8cd03a22f91f2f2fcd5c2c949e4cb7e9`
- `src/ui/room.js` SHA-256 `b4987a0b4b4319c135840ba596d2029d3278dd3535ec28ca1e97374a35dfacc0`
- `tests/claude_usage.rs` SHA-256 `c00fae7b26b8e467fdd296f7ad8b0e9eac2a288b5d99d1b03be3a74cd753a925`
- `tests/startup_quota.rs` SHA-256 `dae4d89618c55bcda1b379c3c3af439c3e54d134164e1158de2b79cf5f93e998`
- `tests/test_quota_badge.py` SHA-256 `600e0cf9279c8d0bacacd223861ed60d0d9bb93180ee76679bfedf87ca0fd663`
