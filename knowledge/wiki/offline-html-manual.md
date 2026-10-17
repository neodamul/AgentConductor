# Offline HTML manual

The Markdown manuals remain the authoring source. Run `target/release/agentconductor manual render` after changing their text or screenshots. Generated English/Korean HTML contains real command tables and self-contained local images; tests reject drift, executable links and escaping the asset directory.

The app fetches the chosen embedded HTML only when its User manual dialog opens. This keeps large screenshot data out of the initial workspace document. DOMParser imports generated body nodes, without scripts or remote HTML. No runtime Markdown dependency or network server is required.

Trace: R-144 / DD-153.

Native screenshot rendering reuses the same Markdown paragraphs and an explicit bundled-asset allowlist. Since 2026-10-17 the public manuals use Rust-native `.jpg` captures as well as PNG workspace screenshots. `src/manual/images.rs` canonicalizes each target under `docs/assets/`, accepts only `.png`, `.jpg`, and `.jpeg`, and embeds the matching `image/png` or `image/jpeg` data URI. Public screenshot crops remove the local absolute path shown in the app status bar. The native eframe viewer has an exact embedded-byte allowlist and verifies screenshot alt text through AccessKit. `image` 0.25.10 is optional in Cargo metadata but enabled by the default `native-ui-probe` feature; its MIT OR Apache-2.0 declaration and the selected Zune JPEG decoder's permissive MIT/Apache-2.0/Zlib declarations and license texts were checked. This supersedes the 2026-10-16 prototype-only note above.
