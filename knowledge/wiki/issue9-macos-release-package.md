# Issue 9 macOS Release Package Verification

## Verified

Source head `6eb2ca9f` built successfully with the locked release profile. The Rust manual renderer generated English and Korean HTML manuals, then `package-release` completed successfully.

The resulting Apple Silicon archive is `agentconductor-aarch64-apple-darwin-6eb2ca9fa8f6.zip`. Its SHA-256 is `52b6fe6294bec4b92e709b206fd82cb7aa3b0011d420f9976a8b481f0092a29e`; the packaged executable SHA-256 is `2abb5df4c8ad03bb0b04ed08635b2f9fd1d7a4eb34b2a655f62b15073f24dcb6`.

The archive contains the `.app` bundle, executable, bilingual HTML manuals, dependency license notices, and a CycloneDX SBOM.

## Scope

This is local macOS package smoke evidence. It does not establish Windows package acceptance or shipped/default Rust UI parity. Retain the distinction when updating issue #9 acceptance.

## Source

See `knowledge/raw/2026-11-02-issue9-local-macos-package.md` and `docs/09-completion-audit.md`.
