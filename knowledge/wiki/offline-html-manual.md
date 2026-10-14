# Offline HTML manual

The Markdown manuals remain the authoring source. Run `python3 scripts/render_manual.py` after changing their text or screenshots. Generated English/Korean HTML contains real command tables and self-contained local images; tests reject drift, executable links and escaping the asset directory.

The app fetches the chosen embedded HTML only when its User manual dialog opens. This keeps large screenshot data out of the initial workspace document. DOMParser imports generated body nodes, without scripts or remote HTML. No runtime Markdown dependency or network server is required.

Trace: R-144 / DD-153.
