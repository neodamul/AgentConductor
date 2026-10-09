# Knowledge asset rules

raw/ stores source observations; wiki/ stores maintained knowledge; this file defines rules.
Pattern: [Karpathy LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f).

1. Store sources/observations with URL/path, retrieved_at, hash, and revision.
2. Preserve copyrighted sources as links, metadata, and permitted short excerpts, not full copies by default.
3. Pages identify sources, verification date, facts, proposals, and unknowns.
4. Connect new sources to existing pages; mark conflicting claims instead of deleting them.
5. index.md lists topics; log.md is append-only. Update both when incorporating ingest/query results.
6. Lessons include context, failure, cause, response, evidence, confidence, scope, and revalidation triggers.
7. Unverified agent claims remain drafts; success reports alone do not justify acceptance.
8. Mark relevant claims stale when source hashes or code commits change.
9. Workers write separate drafts; one coordinator writer promotes canonical content.
10. Keep secrets and approval records outside the wiki. Store progress summaries and run references only.

Self-managed backend: Markdown, Git history, SQLite FTS5. agentmemory provides searchable
replicas and lesson recall. Graphify/CocoIndex indexes are rebuildable derivatives, not sources.
All documents are English; communicate with the user in Korean. README is public-facing and
excludes development plans/internal decisions. The log records this language migration;
Git history preserves the earlier text.
