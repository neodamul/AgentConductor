# Cross-platform development continuity

The canonical source is the private `neodamul/AgentConductor` repository. Development decisions and acceptance work are in `docs/01-requirements.md` through `docs/09-completion-audit.md`; reusable source-hashed assets are in `knowledge/wiki`. Agentmemory is an optional local companion; repository documents remain available without the original host's memory server.

## Resume from another host

1. Clone the private repository using that host's GitHub authentication.
2. Read `docs/07-implementation-plan.md` and `docs/09-completion-audit.md`. A checked checkpoint is not full-product completion.
3. Install Rust with Edition 2024 support, a compatible native C/C++ build toolchain for bundled SQLite, and Git.
4. Configure agent executables, catalogs, permissions and authentication on the new host. Do not copy macOS absolute paths, credentials, runtime databases or verification fixtures into the repository.
5. Follow Plan -> Task (Red/Green/Refactor) -> Verify -> Complete and retain real command evidence and OS-specific acceptance gaps.

## Current Windows boundary

Native Windows runtime is not yet verified or supported. Current Unix-only process groups, ownership probes, leases, secure file identity checks and `/dev/urandom` must be replaced before a working Windows release can be claimed. Native Windows build results must be captured rather than assumed. The native desktop shell currently uses macOS Cocoa/WebKit.

For interim development, WSL can provide a Unix Rust/Git environment and separately authenticated Linux CLIs. Clone within the WSL filesystem, configure Linux CLI paths and use the CLI. This is an interim developer route, not the native Windows release target. Actual WSL end-to-end acceptance is still unverified.

On a supported macOS checkout:

```sh
cargo build --locked
cargo test --all-targets --locked
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Windows native build/test commands are the same Cargo commands from a Developer PowerShell configured with Rust and MSVC. Failures from missing platform implementations must be fixed without weakening containment or approval invariants.

## Release acceptance

Do not ship a Windows artifact merely because it compiles. Test the packaged binary's CLI auth, JSONL, session isolation, cancellation, timeout, descendant cleanup, recovery, repository lease and exact approval/merge flow on the native platform. Include SHA-256 checksums and dependency license notices. Publish verified macOS and Windows assets to the existing private repository's GitHub Release, with explicit desktop support and signing status.

## Native runner validation

`.github/workflows/platform-validation.yml` builds and tests the committed source on `macos-latest` and `windows-latest` with a 20-minute bound and independent platform results. It grants read-only repository access and has no release publication permission. Unix-only tests are inapplicable on Windows; a green build cannot replace future Windows containment/desktop acceptance. The official checkout action is pinned to commit `11bd71901bbe5b1630ceea73d27597364c9af683`, whose MIT license was checked in the upstream repository on 2026-10-10. GitHub-hosted runners are build/verification infrastructure, not a runtime cloud dependency.
