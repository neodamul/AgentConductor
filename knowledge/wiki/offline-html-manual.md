# Offline HTML manual

The Markdown manuals remain the authoring source. Run `target/release/agentconductor manual render` after changing their text or screenshots. Generated English/Korean HTML contains real command tables and self-contained local images; tests reject drift, executable links and escaping the asset directory.

The app fetches the chosen embedded HTML only when its User manual dialog opens. This keeps large screenshot data out of the initial workspace document. DOMParser imports generated body nodes, without scripts or remote HTML. No runtime Markdown dependency or network server is required.

Trace: R-144 / DD-153.

Native screenshot rendering (2026-10-16) reuses the same Markdown paragraphs and only recognizes the six bundled asset paths. `image` 0.25.10 is enabled only by `native-ui-probe`; its MIT OR Apache-2.0 declaration and the selected Zune JPEG decoder's permissive MIT/Apache-2.0/Zlib declarations and license texts were checked. The default app feature stays free of this decoder dependency.
