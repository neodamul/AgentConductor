# Functional Agent Room evidence

Historical HTTP-transport checkpoint, superseded by [Direct native transport](direct-native-transport.md). Verified: 2026-10-10 KST. This is an implementation checkpoint, not full-product completion.

## Observed runtime

The actual embedded interface was browser-rendered against a named disposable local
fixture in `.agentconductor/verification/ui-room-live-v2`. Three compiled mock subprocesses
completed scoped tasks with test evidence. Computer-use interaction submitted targeted
feedback, moved one task from `local-builder` to `local-reviewer`, selected the observed
`fixture-large` model, resumed the affected task, inspected frozen diff/tests, approved
the exact candidate and separately confirmed merge. The authoritative run then became
`completed`; Git integration is also asserted in the backend test. The original failed
fixture input run remains retained as `ui-room-live` instead of being erased.

The fixture implements its fixed contract; it does not demonstrate AI reasoning,
provider model switching or authenticated native permission enforcement. Its task mode
returns fixed content even when feedback requests different content. The screenshot is
real runtime evidence with fixture results, not a fabricated native-provider session.

## Boundary and lessons

- Typed loopback actions run in owned CLI children, with immutable revision checked
  under the workflow lock. Candidate/tool digests retain their separate authorization.
- Cookie names include the port: host cookies are not port-scoped. Red regression
  confirmed independent windows would otherwise replace each other's capability.
- Report APIs omit profile environment values. Bounded Unix artifact reads walk
  directory descriptors and refuse aliases/nonregular files instead of reopening
  previously checked paths. This is an evidence-read boundary, not OS worker isolation.
- Review stays frozen until explicitly refreshed. Feedback makes prior approval stale;
  acceptance is separate from a CLI child's successful exit.
- A new stop marker cannot be discarded by an older resume request. Separate short
  stop-marker locking serializes comparison/removal and atomic stop replacement.
- Whole-suite testing encountered a retained PID-based directory collision. Test UUID
  directories retain previous failed artifacts and avoid PID reuse.

## Evidence and limits

Backend suite: seven tests across UI and stop/resume suites. Whole suite: 142 passed,
four ignored, 39 suites in 55.78 seconds. Clippy clean. The later targeted UI test also
checks wrong merge digest refusal and that closing the server does not cancel accepted
merge; five tests passed in 2.18 seconds. No runtime dependency was added. Graphify
extracted 482 nodes/1,508 edges without an LLM; CocoIndex located both Controllers.
Ponytail guidance was applied; no executable/plugin invocation is claimed.

Cocoa/WebKit launcher compiled and started. Embedded Mach-O Info.plist follows Apple's
[single-file tool metadata guidance](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html)
and [build-setting section contract](https://developer.apple.com/documentation/xcode/build-settings-reference).
The computer-use app inventory initially did not recognize the unbundled executable.
A local app metadata wrapper around the same executable enabled actual Cocoa/WebKit
review rendering and a native screenshot. Cmd-W initially left the window open; adding
standard application/edit/window menus fixed it. The updated wrapped process exited 0
after Cmd-W, and its local endpoint stopped. Bare-executable inventory identity remains
open. Native lifecycle with active workers is covered separately by the backend child
continuation test; it is not yet a native active-worker UI interaction proof.
Browser rendering at native 1280×860 and narrow 715px exposed and corrected clipped
controls. System Cocoa/WebKit is linked, not bundled for redistribution; final release
closure/resource/license checks remain open. Initial goal/profile setup and UI schedule/
tool-gate end-to-end interactions are also open in the completion audit.

## Fixture hashes

- Integrated candidate: `00c139f33a718e84697eea470ac782cf56649db9`
- `.agentconductor/verification/ui-room-live-v2/run/report.json`: `3a6a94cf39e280272bacd739079eb0397ab245b72cbfb3d8b4991ce7d1a9cb1b`
- `.agentconductor/verification/ui-room-live-v2/run/diff.patch`: `99b82e014aaef706a0cde81f93071d218c83085f73eeb8bd1673ea659f3a77ba`

## Source fingerprints

- `src/ui.rs`: `2d0ab9957bf89473f4f1af1c8581abd2d899df48b62c8c90c6a850c818ec405a`
- `src/ui/native.rs`: `00500880ed225ba8a702ffb675308fcc7fbc78e888a295dcc4622773121a9763`
- `src/ui/Info.plist`: `1f63e96938d60adc940e3bf09571934a7484ca59ddbd8341b6cbfa0c55c38bd9`
- `src/ui/room.html`: `1b861537466a3c8d96b8b69b07dcb769cdd82e3b5886550f5d8d57cbbd6ee5f5`
- `src/ui/room.css`: `ebd1c7802e8b5d3dd7676e8209752347320d4f67bc7bf8ad681ec6b2789ba825`
- `src/ui/room.js`: `b31c507b45d5795694ca76602d825bb62ff73b5b3beb62426dfaf5c6e9da1ac8`
- `src/workflow.rs`: `d37be52e2921d511b4aaf57530dc849ca69715e10f4a2d033c06f4121894678f`
- `tests/ui.rs`: `abccea94bc0843d7620edf82221ccd7af36ccc77d9ce3067fbb133e337b7980c`
- `tests/ui_resume.rs`: `05eae4d05383b5ea32cd901b88483650ff4859d7988b89b08229a581b3b49435`
- `docs/assets/ui-agent-room-live.jpg`: `6fd7c9ba6a622ff2b2fa448b3353e43faf7451bb1e173a8badf8505723fdeb1a`

## Verification process cleanup

Direct user correction: close previous owned verification windows/processes and verify
exit before opening a replacement. Store lesson `lsn_5169ce1ac0c2a86d` with confidence 0.7.
The native interface now holds a per-run kernel lease; another CLI process refuses
opening the same run while it is held, and release permits reopening. This does not
terminate coordinators or allow unsafe takeover. All known native/server handles were
terminated or observed exited before final cleanup. macOS pgrep inventory is unavailable
(`sysmond service not found`), so it must not be cited as proof of absence. Native app
inventory and owned handles/endpoints provide the observed cleanup evidence.

A separate availability fixture (`ui-availability-live`) retained three children ending
without an exit code or stdout; a direct single fixture invocation later emitted a
valid usage-limit event. The cause of this subprocess outcome remains unresolved, and
availability UI acceptance stays open. Do not hide it by replacing the retained run.

- `docs/assets/ui-agent-room-native.jpg`: `d979c61985047936e26464020946e4d16bd20bffcc772dd88f0d86af6a51aa0e`
