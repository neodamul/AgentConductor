# Korean input

Trace: R-32 → BG-13 → SF-27 → UC-24 → QAS-25 → DDR-07 → DD-34.

## Contract

Native forms track `compositionstart`/`compositionend`. Polling never assigns repository,
goal or feedback values during composition, and the Cmd+Enter send shortcut ignores
`isComposing` events, so the final syllable is committed before sending. Text is passed as
UTF-8 without normalization.

## Evidence

Korean goals round-trip into the planning prompt and plan. Synthetic keystrokes during
native acceptance were garbled because the host input source was Korean 390 Sebeolsik;
automation therefore uses Edit → Paste, and real composition is verified in the app.

## Native acceptance (2026-10-10)

With the system Korean 390 Sebeolsik input method, virtual key codes (not synthesized
Unicode `keystroke` text, which bypasses the input method) composed `안녕 하세요` in the
workspace goal across live polling; the saved workspace contains the identical UTF-8.

## Source fingerprints

| Source | SHA-256 |
| --- | --- |
| `src/ui/room.js` | `0e2f9eceedd0ae49e25976f00c33a9189f9f6dd43a73c19983f1380590fe42a2` |
| `src/ui/startup.js` | `2f86c27def9a53ef6cc20a0e6df590db4054bbafba6328130862dd27f7290351` |
| `tests/korean_input.rs` | `066fa3200291d51f9913927ac902d0023dc53dfa5e16e64c2684fe2dbd998498` |
