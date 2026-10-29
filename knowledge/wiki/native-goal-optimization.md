# Native goal optimization

## Decision

Goal optimization runs from the native Rust workspace through the selected coordinator's existing CLI profile. It does not add an OpenAI-compatible endpoint, API key, background service, or separate process slot. The bundled prompt-optimizer reference is pinned and distributed with its Apache-2.0 license and provenance.

The coordinator receives a bounded, read-only optimization request. Its proposal is shown in a native review dialog; the saved goal changes only after the operator explicitly accepts it. Keeping or dismissing the proposal leaves the goal unchanged.

## State and request flow

- `src/ui/setup/optimize.rs` validates and dispatches the operator request.
- `src/commands/optimize.rs` runs the coordinator request in the existing startup operation slot.
- `src/prompt_optimizer.rs` bounds and parses the optimizer prompt/result contract.
- `src/native_ui_probe/workspace/optimizer.rs` owns native proposal state and review actions.
- `src/native_ui_probe/worker.rs` requests workspace chat history and optimizer results in the initial snapshot. The Agent Room uses its smaller strict snapshot shape.
- `src/ui/setup/request.rs` defaults optional snapshot fields so older clients and existing chat requests remain valid.

Workspace snapshots carry chat history and optimizer results together. Chat reset behavior remains independent. A saved optimizer proposal is retained only while its repository and goal still match the configuration.

## Verification and limits

The focused offline optimizer, startup, coordinator-chat, package-license, source-layout, and public-document suites passed 53 tests across 13 suites after the snapshot compatibility regression was fixed. Warning-denied all-target Clippy and rustfmt passed locally. Every changed Rust/test file is subject to the 300-line source-layout guard.

This is source-level local verification. A real coordinator invocation, native app screenshot review, and exact macOS/Windows package journeys remain unverified and do not count toward issue #9 packaged acceptance.

## Traceability

Main issue #180 is being ported as part of Rust UI migration issue #9. The implementation is isolated from the JavaScript UI and reuses the existing Rust coordinator and workspace architecture.
