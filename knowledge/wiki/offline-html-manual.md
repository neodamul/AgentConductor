# Offline HTML manual

The Markdown manuals remain the authoring source. Run `target/release/agentconductor manual render` after changing their text or screenshots. Generated English/Korean HTML contains real command tables and self-contained local images; tests reject drift, executable links and escaping the asset directory.

Keep each generated HTML manual paired with its Markdown source during merges. Do not resolve a generated-file conflict by choosing one side while auto-merging the source. Update release-version headings in both source manuals, then run `cargo run --locked -- manual render`; `manual_renderer` verifies generated output and `public_docs` checks required instructions in both formats. On 2026-10-22, mixing newer Markdown with older generated HTML caused version and content drift; regenerating from the v0.1.36 English/Korean sources restored consistency.

The app fetches the chosen embedded HTML only when its User manual dialog opens. This keeps large screenshot data out of the initial workspace document. DOMParser imports generated body nodes, without scripts or remote HTML. No runtime Markdown dependency or network server is required.

Trace: R-144 / DD-153.

## Native table layout regression — 2026-10-23

The native egui manual must allocate readable column width and let wrapped cell content determine row height. A runtime review of the exact v0.1.36 package exposed 48-pixel column spacing and overlapping wrapped rows. The renderer now divides available width across columns, enforces a 220-pixel minimum, and lays out cell segments vertically so each row advances by its full content height. TDD covers both the column spacing and wrapped-row overlap; the exact package UI confirms the command table is readable. Evidence: `docs/evidence/issue9-d051-native-manual-table-package-20261023.json`.

Native screenshot rendering reuses the same Markdown paragraphs and an explicit bundled-asset allowlist. The current public manuals use PNG captures for Workspace and Agent Room. `src/manual/images.rs` canonicalizes each target under `docs/assets/`, accepts only `.png`, `.jpg`, and `.jpeg`, and embeds the matching `image/png` or `image/jpeg` data URI; JPEG support remains available for other reviewed manual assets. Public screenshot crops remove the local absolute path shown in the app status bar. The native eframe viewer has an exact embedded-byte allowlist and verifies screenshot alt text through AccessKit. `image` 0.25.10 is optional in Cargo metadata but enabled by the default `native-ui-probe` feature; its MIT OR Apache-2.0 declaration and the selected Zune JPEG decoder's permissive MIT/Apache-2.0/Zlib declarations and license texts were checked. This supersedes the 2026-10-16 prototype-only note above.


## Current bilingual package assets — 2026-10-23

The v0.1.36 macOS arm64 package at source `575fef74d160d6050a878e89d3bafa7f831432f5` passes `validate-package` with 242 dependencies. Both generated HTML manuals and both Agent Room PNGs have package hashes matching the current repository assets. After discarding an initial wrong-window selection, the exact package was launched with an isolated `HOME` and workspace; its User manual rendered both the dark Korean Workspace screenshot and the English Agent Room screenshot showing three completed tasks/tests awaiting human review. No task, approval, or merge was performed. This adds no workflow credit; acceptance remains 15/25. Windows exact-package verification remains open. Evidence: [`issue9-current-source-package-manual-20261023.json`](../../docs/evidence/issue9-current-source-package-manual-20261023.json).
