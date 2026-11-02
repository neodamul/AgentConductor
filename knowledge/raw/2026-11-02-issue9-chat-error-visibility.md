---
source: exact Windows package run 41380213230, PR #22, and local regression test
retrieved_at: 2026-11-02T13:00:00+09:00
revision: 00b39c5a2297927ea5b3d9de099476212025067c
hash: git commit 00b39c5a
---

# Source observation: Windows packaged coordinator chat timeout

The exact Windows package run failed two chat journeys after the runner denied process breakaway from its host Job Object. The first journey detected and reported the host restriction. The two send-without-wait journeys timed out while looking for a running chat job because the Rust chat state retained the refusal as `uncertain_error`, but the view rendered only the ordinary `error` field.

The Rust chat state now exposes its displayed error by preferring the uncertain outcome message over the ordinary error. The chat view renders that value. A UI-driven regression submits a question, enters the uncertain state, and checks the visible-error accessor. Focused tests pass locally; the exact packaged Windows rerun is pending.

Do not classify the Windows journey as passed or skipped until the exact packaged run confirms that the harness detects the explicit Job Object refusal. Do not relax the refusal guard or treat a missing job as an environment skip by itself.
