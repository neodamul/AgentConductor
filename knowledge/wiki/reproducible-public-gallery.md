# Reproduce the public gallery without operating user workspaces

Source: [fingerprinted gallery](../raw/release-docs-891.json), #891.

`scripts/capture_docs.cjs` assembles the current product UI with deterministic local sample data, reuses one owned browser, checks entry into the intended task screen and cleans its temporary profile. It never starts a signed-in agent or alters the operator's project. Label these as sample-data product renders, not native-window or authenticated-provider evidence; approved design previews remain separate.

Do not infer capture success from a file existing. A disabled navigation tab can leave the previous screen visible; wait for availability and assert the intended screen before saving. Update README asset links so the offline package includes every manual image, then regenerate both Rust-rendered HTML manuals after the final image bytes change.
