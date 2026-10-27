# Native update-source settings

The Rust desktop exposes the update repository under **Settings → Update repository**. The editor
loads and saves through the existing asynchronous `RequestWorker`; it does not start a local web
server or add a separate API credential.

The setting accepts a GitHub `owner/name` or an HTTPS GitHub-compatible URL such as
`https://git.example.com/team/app`. GitHub URLs are normalized to `owner/name`; other hosts remain
HTTPS URLs. An empty value selects the compiled default. The backend validates the value before
persisting it in the existing local preferences file, and the dialog stays open with an error if
loading or saving fails.

Release metadata, archives, and checksums all come from the selected repository's latest release.
The setting does not change the download location, package verification, or update-install safety
checks. Backend validation is covered in `tests/update_source.rs`; the native Settings journey
enters a repository, saves it asynchronously, and reads it back through the same Room bridge.

This ports GitHub #130 to the Rust UI while issue #9 remains open. It does not change the packaged
workflow acceptance count (8/25).
