# Interface message catalogs

All Rust desktop UI wording is addressed by a stable ID. English and Korean UTF-8 JSON shards live under `src/ui/messages/en/` and `src/ui/messages/ko/`; Rust embeds them at compile time and merges each language once into its process-local catalog. Each shard is kept at or below 300 lines and follows a feature boundary.

Add or change an ID in both languages, keep placeholders identical, and reject duplicate IDs across shards. `ui::messages::text`, `in_language`, `render`, and `interpolate` are the lookup APIs. Missing translations fall back to English. Interpolation substitutes only the template; supplied values remain unchanged. Do not translate goals, source documents, model IDs, paths, provider output, or raw logs.

Verification is Rust-only: `cargo test --offline --locked --test messages --test i18n_contracts --test interface_language --test source_layout`. The checks cover matching IDs and placeholders, duplicate detection, source references, language persistence, and the 300-line file limit. No JavaScript or Python localization runtime is used.
