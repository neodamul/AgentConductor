# Update release-note display safety

## Contract

Release notes returned by the update service are untrusted text. The Startup view binds
the note body through `AC_I18N.bind`; the shared localization writer assigns
`textContent`, never `innerHTML`. Empty notes hide and clear the body. The release-note
link precedes the body, and the body remains scrollable with a bounded height and
preserved line breaks.

## Verification migration

The prior Python contract executed a JavaScript fixture through Node. Its replacement is
`tests/update_notes.rs`, which statically verifies the Rust-owned repository contract for
the text-only binding, empty-state behavior, DOM ordering, and scroll layout. This moves
the project-owned verification check into Rust; it does not claim that the JavaScript
desktop UI has been replaced. R-151 / QAS-147 remain incomplete until the UI itself is
Rust and the packaged macOS/Windows applications are verified without JavaScript.

## Source fingerprints

- `src/ui/startup.js`: `c52c625998ae171e673e617bbd90c555bcd88e9f2cacd954bc52ba9e647a4ae8`
- `src/ui/i18n.js`: `4683185baa9ccf2b71d20951a90875649eb9b8d804621e7918acd00d6f18be44`
- `src/ui/startup.html`: `348e57230f733c5084dec999ee993606cbedd2df215d43d934af434154939728`
- `src/ui/startup.css`: `981ef283753a087499c0f7d55f2dc3fbc015ff1e13164c8e24f94a36fe74fec7`
- `tests/update_notes.rs`: `bf1c035047b122614519012ac4e8e4f75c79f28e4cdb4dc78b795d9487debae6`

## Validation

Rust all-target tests excluding the slow repository-wide source-layout scan: 503 passed,
14 ignored, 1 filtered. The 43 remaining Python contracts, Rust formatting, all-target
Clippy with warnings denied, and `git diff --check` passed. The source-layout test was
filtered and is not reported as passing.
