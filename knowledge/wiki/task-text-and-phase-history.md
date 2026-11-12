# Task text and Phase history

Verified 2026-11-12 for #878–#880. Trace: R-210 / SF-198 / UC-202 / FR-204 / QAS-209 / DD-222.

## Behavior

Workspace already had a bounded pre-wrap Goal style, but standalone Room embeds only the shared stylesheet. The existing Goal toggle/text rules now live in that shared stylesheet. A changed Goal resets expansion; ordinary polls keep the operator's chosen state.

Raw-log presentation parses complete JSONL records, preserves metadata and literal path escapes, and displays decoded message line breaks. Plain/malformed tail fragments are unchanged. Source artifacts are never rewritten; large records or expanded representations fall back to their original text.

A rail selection carries Phase and task identity before opening the target. Historical details stay read-only, repeated IDs resolve within the chosen Phase, and selecting a current task restores its editable proposal. Finished Goals default to the final Phase but still permit explicit history browsing.

## Evidence and lesson

[Source fingerprints and Red/Green observations](../raw/task-text-phase-878-880.json) record the regression checks. Goal checks pass9 ×4 screen variants plus4 style comparisons; logs pass the pure formatter and7 ×4 runtime checks; Phase navigation passes7 ×8 active/completed variants and the existing Phase picker check.

Context: global task IDs and current-proposal DOM are insufficient for Goal-wide navigation. Failure: a historical selection searches an absent current-Phase row, or a stale proposal forces the final Phase. Response: resolve Phase first, then task; preserve read-only history and operator expansion state. Confidence: high for local assembled UI behavior. Revalidate when document assembly, Goal context, log formatting or Phase rendering changes. Physical Windows UI acceptance remains separate.
