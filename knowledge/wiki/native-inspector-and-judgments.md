# Native inspector and coordinator judgments

The Rust migration integrates main `e9d1f1ff` (#146 and the first two #149 steps)
without restoring JavaScript, CSS, WebView or Node-based verification.

## Trace identity

The native branch had already assigned main's new IDs to different features.
Preserve both histories with these canonical native traces:

| Capability | Main trace | Native trace |
| --- | --- | --- |
| Inspector folding | R-165 / DD-175 | R-168 / DD-179 |
| Coordinator charter | R-166 / DD-176 | R-169 / DD-180 |
| Judgment visibility | R-167 / DD-177 | R-170 / DD-181 |
| Launch gate | R-168 / DD-178 | R-171 / DD-182 |

Main's earlier step-lessons entry is already represented by native R-167/DD-178.
The imported charter/amendment follow-ups remain planned; this integration does
not claim those later #149 steps are implemented.

## UI and authority

`room::inspector::Inspector` separates the remembered operator choice from
temporary decision-driven opening. New keys open once; refolding an outstanding
decision shows an accessible warning. Reduced motion keeps the warning steady.
Only an explicit toggle writes through the existing preferences store. Errors
leave the in-memory choice usable. Candidate decisions precede task controls.

The coordinator is a selectable count row after tasks and uses the existing
central Activity/Review/Logs/Lessons pane. The same asynchronous worker loads
judgments, integration logs and run lessons. Room snapshots refresh this view
when its judgment count changes; there is no new service or poller. A watched
task finishing without a next running task hands focus to a waiting coordinator.
An operator's explicit selection of completed work is preserved.

`workflow::decisions` derives rows from the existing audit; the launch gate
resolves tools before creating a run. Keep backend additions in the already
split Rust modules, and bound judgment lists to 2,000 rows with visible notice.

## Evidence

Rendered Red tests caught the absent fold control, absent task judgment and a
coordinator count placed before tasks. Focused verification passes 49 tests
across eight suites. The real-worker test loads judgments and verifies refresh
after a new audit record without an agent or external service. All source and
tests remain separate and at most 300 lines. Full-suite, current exact-package
and Windows acceptance remain distinct verification gates under issue #9.

## Regression corrections

Do not force Activity to Review when opening the inspector; the decision controls
come first inside the inspector while the operator's chosen tab stays selected.
Count and judgment rows share one read-only SQLite snapshot, preventing a newer
count from suppressing refresh of an older list during concurrent writes.
Judgment summaries supplement the original tool input/reason; they never replace
those richer facts just because a new response field exists.

The broader checkpoint passed 914 tests, 14 ignored, across 303 suites. After
the last one-line rich-tool-context correction, all ten affected tests and
all-target warning-denied Clippy, rustfmt and diff checks pass. AgentMemory
records: `mem_mvjh2ft1_2f08f38e0bd0`, `mem_mvjhxm23_011f01ecb05d`.
