# Scoped agentmemory discovery (DD-44)

Sources: `src/memory/checkpoints.rs` (`address_scope`, `discover`), `src/memory/client.rs`
(`list_scope`), `src/commands/memory_ui.rs`, `tests/cooperative/memory.rs`.

- Contract verified in the installed `@agentmemory/agentmemory` 0.9.29 source:
  `GET /agentmemory/memories?agentId=…&latest=true&limit=50` returns
  `{memories:[{id,project,agentId,content,…}],total,offset,limit}`.
- Discovery filters by the exact run project and address-digest agent tag, then calls the
  unchanged exact-ID `import`; directives are counted and never imported; already imported
  IDs are skipped by receipt.
- Verified against the real local daemon (127.0.0.1:3113): new report imported, two
  previous observations skipped, directive excluded.
- Lesson: read the installed server's route source before depending on an API; similarity
  search is not provenance.
