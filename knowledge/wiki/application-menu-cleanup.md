# Application menu entry points

Verified 2026-11-12 for #865–#867. Trace: R-210 / SF-198 / UC-202 / QAS-209 / DD-222.

## Current behavior

The Workspace group keeps recent, New, Open, Export and Import actions. Open Workspace settings with step 1. Register agents with Add on the left Agents tab. Open or close the left and right panels with their titlebar buttons. The duplicate Agents and View groups and Workspace settings menu command are removed. Plan rules, Messenger, Settings, Help and Close remain unchanged.

## Evidence

[Source observations and fingerprints](../raw/menu-cleanup-865-867.json) identify the tested code. Each obsolete entry failed its new runtime expectation before removal. The final shared Workspace/Room suite checks group structure, keyboard/focus navigation, actual agent registration, Workspace settings visibility, both Room panel toggles and state restoration. It passes 41 checks in each light/dark × 840/1280 variant. Thirty-four focused Rust tests cover translation keys, architecture, source layout, manual generation and existing Workspace menu behavior. The canonical mock verifier passes 157 interactions per variant, 72 captures and 34 contrast pairs.

## Lesson and limits

Context: simplify a shared navigation surface without losing functions. Failure risk: deleting a redundant menu command can also remove its sole live handler. Response: keep the original rail, stepper and titlebar handlers; test those real entry points after deleting menu-only callbacks and translations. Confidence: high for assembled browser behavior; an installed Windows package was not exercised for this unit. Revalidate when shared menu routing, translation keys or step navigation change. See [approved Goal design](approved-goal-ui.md).

Integration with main `c1ffa8d5` preserves the Help bug-report action and Claude's role/state/model tags. The bug-report runtime suite passes eight checks per screen variant alongside the menu suite.
