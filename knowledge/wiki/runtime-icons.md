# Runtime provider icons

Trace: R-34 → BG-14 → SF-29 → UC-26 → FR-20/NFR-10/NFR-12 → QAS-27 → DDR-10 → DD-40.

Provider artwork is resolved from operator-installed applications at runtime; no
provider icon is bundled or copied into this wiki. macOS resolves bundle identifiers
through NSWorkspace and renders a bounded PNG. Windows searches per-user executable
locations and extracts an icon. Built-in providers receive image avatars only when
resolution succeeds; custom providers and missing applications retain letter avatars.
The process caches the results. PNG data URLs are capped at 64 KiB. Native eframe
snapshots carry those URLs only when requested; the sidebar decodes them with the
existing image dependency, caches egui textures by provider URL and falls back to
monograms when decoding or discovery yields no icon.

On 2026-10-10, all five icon tests passed, including the normally ignored test
against actually installed Claude, Codex and OpenCode macOS applications. The
encoded-PNG contract checks chunk CRCs, size bounds and malformed pixels. Native
page visibility remains supported by the earlier Claude acceptance record; this
run verifies extraction and encoding rather than a new visual interaction.

On 2026-10-18, a native rendered-widget regression failed before sidebar artwork
rendering and passed after adding the bounded decoder and texture cache. The focused
suite passed 22 tests; one installed-app icon test was ignored. An optimized macOS
Rust build showed the installed Claude and Codex icons in actual sidebar cards while
a custom agent kept its monogram. The UI snapshots explicitly request icons. The
Windows app was not available for runtime confirmation, so Windows path discovery
and eframe rendering remain unverified.
Local extracted PNGs and their hashes are retained outside tracked assets in
.agentconductor/verification/icons/manifest.json. Native Windows executable
path discovery, extraction and rendering remain unverified; do not infer them
from successful Windows compilation or the macOS icon tests. Revalidate on new
provider install locations, bundle identifiers or operating-system versions.

Sources:
- [Provider resolution and bounded PNG data URLs](../../src/icons.rs).
- [Native texture rendering](../../src/native_ui_probe/workspace/sidebar/icons.rs).
- [Decoder and rendering contracts](../../tests/icons.rs) and [native rendered-widget regression](../../tests/native_ui_sidebar_icons.rs).
