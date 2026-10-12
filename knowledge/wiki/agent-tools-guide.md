# Agent tools guide

Settings → Agent tools guide… is a static dialog listing install commands for agentmemory, ponytail, rtk, graphify and cocoindex-code for Claude Code and Codex. The app never installs or runs them. Commands sit in code elements, which the i18n walker never translates; the user manual section 1.2e mirrors the list.

Commands were taken from each project's README on 2026-10-12. When a tool changes its install path, update startup.html, docs/user-manual.md and tests/test_agent_tools_guide.py together.

Trace: R-81, DD-89, QAS-76. Evidence: tests/test_agent_tools_guide.py.
