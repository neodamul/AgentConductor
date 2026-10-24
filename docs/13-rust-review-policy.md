# Rust review policy

Review guidance for new and changed Rust code, selected from the
[Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html) after an
applicability review (GitHub #169). Trace: R-174 → BG-129 → SF-162 → UC-165 →
FR-160/NFR-70/C-06 → QAS-171 → DDR-34 → DD-185.

The guide itself separates general guidance from Chrome- and Fuchsia-specific rules and
calls its API-design advice binding for libraries but only a recommendation for
applications. AgentConductor is a single application executable, so the policy keeps the
rules that protect safety and readability and drops the rest.

## What stays authoritative

The checks below already enforce much of the guide and are not duplicated here:

- `cargo fmt --check` with the default rustfmt settings (`max_width = 100`).
- `cargo clippy --all-targets --locked -- -D warnings` (default lint groups). These already
  cover `ptr_arg`, `borrowed_box`, `should_implement_trait`, `new_without_default`,
  `len_without_is_empty`, `wrong_self_convention` and `unnecessary_unwrap`.
- `tests/source_layout.rs`: no authored file above **500 lines** on `main`, and no test body
  under `src/`. The Rust migration line additionally keeps every changed Rust file at or
  below **300 lines**; this policy changes neither limit. Entries elsewhere in the
  documents that say 300 refer to that Rust-line limit.

## Review rules

Applied to new and changed code in review. Existing code is not swept; a touched area is
brought in line only when the change needs it. A justified exception is written in the
pull request.

1. Keep the rustfmt defaults and write statements rustfmt can format; split long string
   literals instead of leaving a block that rustfmt silently skips.
2. Add no glob imports except `use super::*` in split child modules and `tests/unit`,
   `pub use child::*` facade re-exports, and `use Enum::*` inside an enum-focused function.
3. Borrow instead of cloning owned data; deserialize JSON with `T::deserialize(&value)`, not
   `from_value(value.clone())`.
4. Take `&[T]`, `&str`, `&T` or `Option<&T>` in parameters, never `&Vec`, `&String` or
   `&Option`.
5. Return `Result` for recoverable external failures. Panic only on a broken invariant, with
   `expect("<invariant>")` instead of a bare `unwrap()` in `src/`; tests are exempt.
6. Prefer `let … else`, `if let` and `match` over checking and then unwrapping.
7. Use an enum, or named fields in a struct, instead of a second `bool` parameter.
8. Prefer named-field structs over tuple aliases and tuple structs; use a newtype when
   values of the same type are easy to mix up.
9. Derive std traits where possible and implement std traits (`FromStr`, `Default`,
   `Display`) instead of same-named inherent methods.
10. Define a trait only for generics, `dyn` dispatch or an extension trait (named `…Ext`),
    never to organize code.
11. Write `unsafe` only for FFI or operating-system calls. Keep each block minimal behind a
    safe wrapper in a platform module, and put a `// SAFETY:` comment on every new or
    changed `unsafe` block and `unsafe impl`.
12. Use `catch_unwind` only at FFI or callback boundaries, where unwinding would cross an
    `extern "C"` function or an evaluator boundary.
13. Never recover data from a poisoned mutex that guards invariants; map the poison to an
    error. Recovering a mutex that guards no data is acceptable with a comment.
14. Document non-obvious `pub` items, panic conditions and `unsafe` invariants. Comments say
    why, not what.
15. Add `#[must_use]` only where ignoring the result is a bug.
16. Build strings from more than two pieces with `format!` or `write!`, not `+` or
    `str::replace` on compile-time templates. Replacing a runtime user template stays valid.

## Explicit exclusions

- Bans on `Arc`, `Rc`, `async` and threads: the application shares state across threads and
  is built on tokio, and the guide's own bans are Chrome-specific.
- One module per `use` statement: it needs the unstable `imports_granularity` option and
  would rewrite about 120 existing nested imports.
- No APIs in non-leaf modules: it contradicts the responsibility-based facade split of
  DD-41.
- Flat file layout, small crates, `chromium::import!`, gtest interop and platform
  synchronization APIs: Chrome- or Fuchsia-specific.
- Unstable rustfmt options (`hex_literal_case`, `format_macro_matchers`) and unstable
  language features.
- `#[non_exhaustive]` and `exhaustive_enums`: a single-team application crate.
- Library-only advice: `IntoIterator` and `impl Into` generalization, `missing_docs`.
- Retroactive sweeps and mechanical rewrites of existing files.

## Measured baseline

Counts are text searches on `main` at `b7fa27a9` (`src/` about 36,500 lines, `tests/` about
28,000); rustfmt and Clippy were not re-run for them.

| Subject | Measurement |
|---|---|
| Glob imports | 86 in `src/`, of which 70 are `use super::*` in split child files |
| `.clone()` | 458 in `src/`; 27 of them are `from_value(x.clone())` |
| `unwrap()` / `expect(` in `src/` | 77 / 9 |
| `unsafe` blocks in `src/` | 125 in 17 files; 3 carried a `// SAFETY:` comment before #172, every one does since |
| `catch_unwind` | 8, all at FFI or evaluator boundaries |
| Lines over 100 columns | 611 in `src/` and `tests/` (characters); 38 in `src/workflow/execution/worker.rs` |
| Public items with `///` | 171 of 573 |

## Executable checks

One check is added beyond the existing formatter, Clippy and layout guard (GitHub #172, extended
by #176): a CI step on macOS and Windows that denies `clippy::undocumented_unsafe_blocks` for the
library, binaries and integration tests (`cargo clippy --lib --bins --tests --features
native-ui-probe -- -D clippy::undocumented_unsafe_blocks`), so rule 11 cannot regress anywhere in
the repository. Tests change the process environment only through `tests/support/env.rs`, a lock
that is held for the whole test and restores the originals on drop; it carries the single safety
comment. Its documented limit is that it serializes mutating tests only, so full soundness needs
one process per test.

No other lint is enabled. Each remaining candidate would fail on many existing sites:
`wildcard_imports` (about 74) and `unwrap_used` (77 in `src/`).

## Porting

The Rust migration line (`rust`, `codex/issue-9-native-ui-parity`) carries its own layout
test and documents. This policy applies there unchanged; port the document and the
traceability entries with a `Related to #169` pull request.
