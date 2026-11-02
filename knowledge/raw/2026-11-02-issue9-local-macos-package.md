# Issue 9 Local macOS Package Evidence

- Source commit: `6eb2ca9f`.
- Release command: `cargo build --release --locked`, Rust `manual render`, then `package-release`.
- Result: successful local Apple Silicon package build.
- Archive: `dist/agentconductor-aarch64-apple-darwin-6eb2ca9fa8f6.zip`.
- Archive SHA-256: `52b6fe6294bec4b92e709b206fd82cb7aa3b0011d420f9976a8b481f0092a29e`.
- Executable SHA-256: `2abb5df4c8ad03bb0b04ed08635b2f9fd1d7a4eb34b2a655f62b15073f24dcb6`.
- The archive includes the app bundle, executable, English and Korean HTML manuals, dependency license notices, and CycloneDX SBOM.
- This confirms local macOS package smoke only. It does not confirm Windows package acceptance or shipped/default Rust UI parity.
