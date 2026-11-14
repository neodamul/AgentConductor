# 0.2.12 ad hoc regression fixes (#985–#988)

## Plan

Preserve the existing update, workspace and notification use cases. Fix shared callers rather than adding an orchestration layer or dependency. Keep operator workspaces and credentials untouched. Each issue has an associated development branch; related fixes are validated together.

## Changes and targeted verification

- #987/#988: one in-flight notification test, disabled duplicate submission, translated pending/success status inside the modal, obsolete error removal, and correct button restoration when settings save overlaps. The new regression first failed on duplicate requests, then passed in both themes, including both languages and save overlap. Existing notification-save checks passed 24/24.
- #986: cancellation before dispatch is observed during unrelated-action and metadata waits. An already dispatched creation disables Cancel and prevents the first Escape. Chromium makes a repeated Escape noncancelable; if it closes the dialog, a persistent notice explains that creation continues in the background. Completion or failure replaces that notice. The regression first reproduced post-cancel creation, then passed queued, metadata, Escape, duplicate-create and post-completion retry checks in both themes. Existing creation-busy behavior also passed. A trusted keyboard probe reproduced the second-Escape close and verified the continuing-work notice; the permanent regression covers forced close and notice replacement after completion.
- #985: macOS bundles restart through /usr/bin/open with LaunchServices; bare executables keep direct launch. Arguments are separate values, and the old-process PID is explicitly forwarded with --env. A rejected launcher handoff returns an error instead of terminating the caller. Four unit checks cover routing, whitespace/Unicode arguments, bare executables, launch failure and nonzero handoff status. The routing assertion failed before the behavior change and then passed. Existing update integration tests passed 4/4.

## macOS handoff probe

A disposable Rust executable in a private temporary app bundle was launched through the same system command. The launcher exited zero in approximately 0.055 seconds; the child received the expected argument and PID environment value. The child exited and the fixture was removed. A second isolated probe used the actual new restart module and the built product binary with an empty temporary HOME/workspace. LaunchServices registered its distinct app identity; native accessibility discovery returned a standard window and HTML content showing the expected version. Only the owned fixture process was terminated and its folder removed. The operator application was not stopped or replaced.

## Limits and privacy

An initial read-only native-window comparison timed out during automatic permission review; the permitted retry succeeded. The isolated new-code native handoff was verified separately as described above. A complete network-download update of the operator application has not been repeated with the new code and remains a release check. No live messenger message, paid CLI session, personal setting or credential was used by these regressions. Only generic data is committed. Full local and batched platform validation results are tracked on the linked PR before merge.

## Batched CI follow-up

Windows passed at 6f42bd42. macOS passed three design cells but failed the dark/1280 deletion-button hover measurement: its color was sampled before the separate hover-state check. Measure hover membership and computed style in the same browser evaluation, retry against the current element, and fail explicitly if hover never settles. Product CSS and comparison tolerances are unchanged. The intentionally static workspace title (pointer-events:none) retains its color comparison without pretending it is an interactive hover target. A small regression first returned an unhovered sample, then passed after the atomic measurement fix. All 16 related comparisons passed across light/dark and 840/1280 widths; all 119 top-level Node UI scripts, the 47 Python checks and formatting check also passed. Final platform results remain required before merge.
