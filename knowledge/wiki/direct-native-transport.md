# Direct native transport

Verified: 2026-10-10 KST. Status: reviewed implementation checkpoint, not full-product completion. Trace: DD-23 and R-17.

## Design and boundaries

The default macOS Room loads embedded HTML/CSS/JavaScript with `loadHTMLString:baseURL:`. JavaScript submits typed JSON strings to `WKScriptMessageHandlerWithReply`; Rust returns serialized JSON through the native reply block on the main queue. No UI TCP listener, cookie, session URL, browser fallback or `ui-serve` command remains. Optional delivery to an existing agentmemory daemon is a separate memory adapter, not a UI server.

Controller operations are bounded to 64 KiB and restricted to snapshot, evidence, task and typed action requests. The owned WebView main frame and embedded `about:blank` document are required. External navigation is refused; CSP disables network connections and permits only nonce-bearing embedded script/style resources. At most eight requests/replies are outstanding. Rust work executes off the Cocoa main thread. Exact review revisions, candidate digests, owned coordinator children and the per-run window lease are unchanged authorities.

## TDD and observed evidence

The first native-request test failed because Room did not exist. Green implementation then revealed that serde internally tagged unit variants could ignore unknown fields despite deny_unknown_fields. Empty struct variants and explicit unknown-field assertions cover snapshot/evidence/resume/recover. Targeted CLI/UI/native/stop-resume tests passed (10 tests); the full suite passed with four explicit native/helper ignores before the final bounded-reply and frontend request refactor, followed by targeted checks and Clippy on final source. No new runtime dependency was added.

Actual Cocoa/WebKit computer-use interaction in `ui-native-ipc/run-v2` sent scoped feedback, resumed the task, inspected frozen diff/test evidence, approved the exact candidate, then confirmed merge separately. Authoritative report state is completed; repository HEAD equals candidate `5d28edd254d5a4acf2bec806b8d54c3aec57e41d`; three verification exit codes are zero. This compiled mock contract returns fixed content, so it proves UI/control integration rather than AI reasoning or provider authorization.

The owned native PID 87549 was successfully inspected with lsof (57 descriptors, query exit 0). Its separate TCP LISTEN query returned no descriptors, no stderr and exit 1. Cmd-W closed the app and its owned process exited 0. After the final frontend request cleanup, the final compiled Room (PID 93424) rendered authoritative completed state and delivered second-task stdout/stderr through the native bridge, then also closed with exit 0. The same executable used an ignored app metadata wrapper for computer-use identity; bare-executable inventory identity and final release packaging remain open.

Graphify structural extraction: 489 nodes / 1,543 edges, no model/API key. CocoIndex located the native callback. Ponytail has no callable tool in the current session; previous recorded guidance is not described as plugin execution.

## Retained failure and limits

The previously built fixture executable produced empty logs and no numeric exit code in the first native fixture run. The failed run is retained in `ui-native-ipc/run`; compiling the current test mock to a new executable enabled the separate successful `run-v2`. The original availability fixture failure remains open. Do not infer its signal or cause from exit=None.

Initial goal/profile setup, schedule/tool-gate native interaction, all-provider native authentication/containment, resource measurement and release licensing/closure remain open in the completion audit. macOS 11+ supports the native shell; other platforms use the CLI.

## Source and artifact fingerprints

- `src/ui.rs`: `e47d1b52056cc542fbc2d997c2b76f79b782f1f2da7a707c658429f4bbc1c8c4`
- `src/ui/native.rs`: `f0bf327f5131bf00db52245ee985630624048af54ba3c25e8c1df4c5a530f29d`
- `src/ui/room.js`: `58c680ad976f1c7c3cc013e558ec29b2ebe4138895ff6bb83b84af3170a51393`
- `tests/native_room.rs`: `d2af0427cd28c0f16ddf57479d0a34b71c8e42531ec1c80c542cec3fb5adf42f`
- `tests/ui.rs`: `af4be7857bf48b72b7b70130c9212874c8ff3f48781c9cb50c7219ca7a91b57f`
- `tests/cli.rs`: `97d6181d284e751499d2f86a9e98c9926083e7006c976731756d17781ccb236f`
- `docs/assets/ui-agent-room-native.jpg`: `b4e90687d76d241a9752babe2e712c9bf8a5bba0dcdd56430dae98ca44ced1f3`
- `.agentconductor/verification/ui-native-ipc/run-v2/report.json`: `baceb0526072529a9e2bc28ed783c064d9882a61d508e217d7ecd8db3020cea7`
- `.agentconductor/verification/ui-native-ipc/run-v2/diff.patch`: `99b82e014aaef706a0cde81f93071d218c83085f73eeb8bd1673ea659f3a77ba`
- `.agentconductor/verification/ui-native-ipc/no-listener.json`: `e6a04f605d6ef9d6685ab846fd3b664d956612d4db9108f7e1b0022e658716e6`
