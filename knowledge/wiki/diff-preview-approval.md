# Diff preview and approval workflow

User priority: shorten the preview with an ellipsis and CLI instruction; permit approval.

Plan: retain the 512 KiB presentation bound, hash the complete patch from the same opened artifact, preserve exact candidate/revision checks, and keep the CLI notice next to the omitted section. Display truncation changes presentation only.

TDD: first fail the large-patch tail-tampering native test and the truncated-preview approval runtime test; then implement and verify. Quota/DD136 work remains preserved separately.

Verification: Red failures recorded in AgentMemory; native full-patch tampering regression passes, Python 64 tests pass, all-target Clippy with warnings denied and app build pass. Complete requires own-file commit/main push and native restart.
