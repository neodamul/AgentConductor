# 0.2.12 ad hoc regression fixes (#985–#988)

## Plan

Preserve the existing update, workspace and notification use cases. Fix shared callers rather than adding an orchestration layer or dependency. Keep operator workspaces and credentials untouched. Each issue has an associated development branch; related fixes are validated together.

## Changes and targeted verification

- #987/#988: one in-flight notification test, disabled duplicate submission, translated pending/success status inside the modal, obsolete error removal, and correct button restoration when settings save overlaps. The new regression first failed on duplicate requests, then passed in both themes, including both languages and save overlap. Existing notification-save checks passed 24/24.
- #986: cancellation before dispatch is observed during unrelated-action and metadata waits. An already dispatched creation stays visible and cannot be dismissed as canceled. The regression first reproduced post-cancel creation, then passed queued, metadata, Escape, duplicate-create and post-completion retry checks in both themes. Existing creation-busy behavior also passed.
- #985: macOS bundles restart through /usr/bin/open with LaunchServices; bare executables keep direct launch. Arguments are separate values, and the old-process PID is explicitly forwarded with --env. A rejected launcher handoff returns an error instead of terminating the caller. Four unit checks cover routing, whitespace/Unicode arguments, bare executables, launch failure and nonzero handoff status. The routing assertion failed before the behavior change and then passed. Existing update integration tests passed 4/4.

## macOS handoff probe

A disposable Rust executable in a private temporary app bundle was launched through the same system command. The launcher exited zero in approximately 0.055 seconds; the child received the expected argument and PID environment value. The child exited and the fixture was removed. The operator application was not stopped or replaced.

## Limits and privacy

A read-only native-window comparison could not complete because automatic permission review timed out. The new code has not performed a live update of the operator application; native visual verification remains a release check. No live messenger message, paid CLI session, personal setting or credential was used by these regressions. Only generic data is committed. Full local and batched platform validation results are tracked on the linked PR before merge.
