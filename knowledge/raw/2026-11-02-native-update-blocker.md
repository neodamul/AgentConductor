# Source observation: native update blocker port

- Retrieved: 2026-11-02 KST
- Source revision: AgentConductor main `76d0ac7d83a5beb16f8813065e5671b0799d70ab` (#332, #331)
- Rust port revision: `a86ec1032cc5c1fc58b329369328924c93e76438`
- `src/ui/update_guard.rs` SHA-256: `3119c1faf39311cec11f4bbeb7f03519b875bc1f277f23815933f78f950f9050`
- `src/ui/setup/update_blockers.rs` SHA-256: `a2e3196097ea8fede28c06ce65c674b41ee58dbec76983057bdd6ac9dbe7cf47`
- `src/ui/messages/en/native.json` SHA-256: `b658380624bfedee8113eddad2d0dfb06e51f78acab135dc79da26bea2a56268`
- `src/ui/messages/ko/native.json` SHA-256: `40ff645edb8a03b5898d6c144a54970a3e08bb68e7b1f121ccae54a1811a7b6f`

The Rust branch ports the current main guard and native refusal UI. Local focused tests, formatting, and warning-denied Clippy passed. Hosted exact-head macOS and Windows validation was queued when recorded. This does not establish packaged workflow acceptance or full UI parity.
