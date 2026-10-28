# Native UI Interaction Tests

Native UI component tests should exercise the action a person triggers, not only verify that an accessible button is rendered. Enable AccessKit in the egui context, locate a button by its localized accessible label, and dispatch pointer-move, press, and release events at the button's reported bounds. Assert the resulting domain action, clipboard command, or dialog state.

For packaged Windows acceptance, UI Automation `InvokePattern` may be attempted when a control refuses keyboard focus, but a returned call does not prove activation. Wait for an observable UI state change; if the control remains enabled, use a foreground- and process-checked pointer fallback. A successful API call, emitted input diagnostic, or visible button is not proof that an action ran. Keep keyboard and pointer fallbacks guarded by exact process/control identity; never credit the workflow unless its durable state matches the requested value.

When testing both English and Korean, use the real localized labels and run the same interaction in each language. For asynchronous panels, cover both accepted actions and the pending state: repeated work must remain disabled while safe navigation such as Close stays available.

Every synthetic egui frame returns a `FullOutput`. Tests that do not apply texture updates must clear both `textures_delta.set` and `textures_delta.free` before dropping the output; clear intermediate frames inside the click helper and the final frame in the caller. Otherwise egui panics on an unapplied texture delta and obscures the actual interaction result.

Issue #9 coverage added this pattern to the manual and updater panels. The component regressions verify copy/close in both languages, check/apply/close in both languages, and duplicate-action blocking while an update request is pending. These tests do not count as packaged end-to-end workflow acceptance.

## Popup menus that contain editable controls

egui `Popup::menu` defaults to `CloseOnClick`, which also closes the popup when a user clicks
inside it. A workspace menu contains a name field, so use `CloseOnClickOutside` and explicitly
close the popup after a workspace action, confirmation, guide, or manual opens. Test the full
pointer-to-text-to-Enter interaction; merely asserting that a text input is present misses this
failure. The Settings menu remains an ordinary `menu_button` because its items are selections,
not an editable form. Rendered geometry tests verify that opening either top-bar menu preserves
the underlying page position.

### Keep menu geometry tests locale-independent

Read the visible/accessibility label from the component's localization state instead of hard-coding
English or Korean. CI may run with a different default language than a developer machine; the
geometry assertion must still locate the same control. For example, use
`State::text("startup.settings")` for the Settings menu button and
`State::text("startup.language_korean")` when asserting that the language choice is visible.

### Match visual evidence to the executable

An egui geometry test proves behavior of the tested source, not of an already running packaged
application. Record the executable path and build time (or commit manifest), and ensure the running
process was built from the same source before claiming a screen-level fix. Keep a real workspace
window intact when it may contain unsaved state; use an isolated profile for visual verification.

## Review detail popups

Whole-goal coordinator analysis, phase task details and editors, the Agent Room briefing, and
advanced agent-profile rules belong in anchored egui popups, not inline `CollapsingHeader`s, so they
cannot move surrounding controls. Bound detail popups and make long content scrollable. egui
popups may omit plain child text from the AccessKit update;
attach explicit accessible names to popup controls and task-row content, assert those names in the
accessibility tree, assert popup text from rendered `Shape::Text` output, and compare phase/action
bounds before and after opening. This verifies accessibility, visibility, and stable geometry.

MCP source-import review cards follow the same rule: open each full source in an anchored menu popup,
with a bounded scroll area. Scope repeated per-record controls with the reference key using
`push_id`; otherwise identical labels can collide and route input to the wrong card. The regression
test clicks a specific source, confirms its body is visible, and verifies that the next record keeps
its position.

The source-selection preview is a longer detail view, so render it in a centered, scrollable window
instead of inside the Workspace scroll area. A rendered regression measured the execution-control
anchor moving from y=61 to y=177 when the preview list was inline; after moving the list to its own
window, the anchor stays fixed. Audit every `menu_button` and inline conditional when the user finds
another reflow: anchored menu popups and centered windows overlay the page, while a result list
painted directly in the page can still change its geometry.

Long-form User manual and Agent tools guide content, workspace-action confirmations, execution
consent, and update details should use centered, stable-ID egui windows, not inline frames after
the navigation bar or in the page body. In headless rendering tests, put the root content in a
`CentralPanel` and settle the window for two frames; compare a workspace anchor before and after
opening and assert rendered text/controls. This catches page reflow without relying on a screenshot.

Editable task-detail popups must use `PopupCloseBehavior::CloseOnClickOutside`; otherwise clicks on
text fields or selectors dismiss the editor. Test field interaction as well as popup visibility and
stable bounds.

Keep geometry and accessibility suites for native detail popups in the platform-validation
workflow's native UI test and Clippy target lists. A local all-target run alone does not cover feature-gated
`native-ui-probe` tests on macOS and Windows; assert workflow coverage in `tests/release_notes.rs`.

Cargo auto-discovers integration tests even when the native UI feature is disabled. Add a crate-level
`#![cfg(feature = "native-ui-probe")]` to every UI-only integration-test file, and keep the
no-default-feature target audit in the platform workflow. This prevents macOS/Windows builds without
the opt-in UI feature from compiling UI-only test bodies against feature-gated library modules.

### Keep PowerShell `Add-Type` C# compatible

Windows PowerShell `Add-Type` may use a legacy C# compiler. Avoid newer syntax such as `out var`
in embedded helper code; declare the output variable with an explicit type before passing it with
`out`. Add a source-contract regression test so this failure is caught on non-Windows hosts before
the exact-package UI run.
