# Runtime provider icons

Trace: R-34 → BG-14 → SF-29 → UC-26 → FR-20/NFR-10/NFR-12 → QAS-27 → DDR-10 → DD-40.

Provider artwork is resolved from operator-installed applications at runtime; no
provider icon is bundled or copied into this wiki. macOS resolves bundle identifiers
through NSWorkspace and renders a bounded PNG. Windows searches per-user executable
locations and extracts an icon. Built-in providers receive image avatars only when
resolution succeeds; custom providers and missing applications retain letter avatars.
The process caches the results. PNG data URLs are capped at 64 KiB.

On 2026-10-10, all five icon tests passed, including the normally ignored test
against actually installed Claude, Codex and OpenCode macOS applications. The
encoded-PNG contract checks chunk CRCs, size bounds and malformed pixels. Native
page visibility remains supported by the earlier Claude acceptance record; this
run verifies extraction and encoding rather than a new visual interaction.
Local extracted PNGs and their hashes are retained outside tracked assets in
.agentconductor/verification/icons/manifest.json. Native Windows executable
path discovery, extraction and rendering remain unverified; do not infer them
from successful Windows compilation or the macOS icon tests. Revalidate on new
provider install locations, bundle identifiers or operating-system versions.

Sources:
- [src/icons.rs](../../src/icons.rs): SHA-256 `29473114092662cff0a2da8870bf4f7d6209105275024d0e50240bc5f97ad301`.
- [tests/icons.rs](../../tests/icons.rs): SHA-256 `68c05979a8f7642ce752166c3d2fbffc6f22895ee92fd48eacfd8618d56b3762`.
- [src/ui/startup.js](../../src/ui/startup.js): SHA-256 `ea873078f3bac5d1f7114aea8d07f9735b539636653a6633f85a926c54ac6969`.
- [src/ui/room.js](../../src/ui/room.js): SHA-256 `ae66c80564d4db7322064e9054c7dfcb1e6afc3a5eb6e2670484ce92fa4351eb`.
