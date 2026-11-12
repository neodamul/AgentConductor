# Browser-based bug reporting

Verified: 2026-11-12. Implementation: #862 / DD-231; PR #864.

## Contract

Help → Report a bug opens a new issue draft in the default browser. Updates and bug reports read the same GitHub repository preference on every request. An empty preference uses the public project repository; an HTTPS Enterprise repository uses its own host. GitHub handles login and the operator edits and submits the issue.

The fixed English template contains summary, reproduction steps, expected and actual behavior, app version, OS and architecture. No workspace paths, goals, logs or credentials are attached. Opening the browser is not successful issue submission.

## Evidence and limits

[Source observation and fingerprints](../raw/bug-report-862.json) record code and tests. Rust normalization, encoding, updated preferences and invalid request tests pass. Shared Workspace/Room menu tests pass in light/dark at 840/1280 widths. Native macOS opening was exercised; authenticated Safari showed the populated composer and an anonymous browser preserved the draft through the login redirect. No issue was submitted.

Physical Windows browser launching and internal Enterprise authentication remain unverified. CI compilation is separate from those acceptance checks.

## Lesson

Context: reuse an existing repository preference without collecting another token. Cause of potential exposure: allowing page-supplied navigation URLs or automatically uploading diagnostics. Response: construct a validated HTTPS URL natively, reject unknown request fields, percent-encode a fixed template and leave submission to the operator. Confidence: high for routing and public GitHub login; limited for untested enterprise policies. Revalidate if repository normalization, native request routing or GitHub issue-composer behavior changes.

See [native transport](direct-native-transport.md) and [knowledge rules](../schema.md).
