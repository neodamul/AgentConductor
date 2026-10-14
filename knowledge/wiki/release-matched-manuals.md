# Release-matched bilingual manuals

The English and Korean manuals share command and JSON identifiers; only explanations and labels are translated. The public README links both manuals and every screenshot they use. Existing release packaging copies those README-linked files, so no documentation dependency or separate runtime is needed.

A version bump must also reconcile provider permissions, direct ordinary-folder execution, automatic recovery, workspace deletion and setup tabs. Do not reuse screenshots from an older version or edit version text into an image. Capture the native app in isolated workspaces, preserve the operator's language preference, and state when completed-run images use offline fixtures.

Windows v0.1.15 compiled successfully but its UI goal-save acceptance failed. The deleted `delete-new-name` field was still referenced during startup, preventing the remaining script from initializing. Removing that reference and checking static element IDs made the original Windows acceptance pass in v0.1.16. A speculative UIA focus patch was preserved but not applied; distinguish a build failure from an executable UI failure.

Tests must compare translated errors through the message catalog, not English fragments. Both manuals must cover every command from executable help; screenshots must exist and be README-linked. Keep development plans and internal decisions in design documents and this wiki, never in the public README.

Sources:
- [Native evidence](../raw/2026-10-14-v016-native-documentation.json)
- [English manual](../../docs/user-manual.md)
- [Korean manual](../../docs/user-manual.ko.md)
- [Documentation checks](../../tests/public_docs.rs)
- [Startup element regression](../../tests/startup_element_ids.rs)

Current screenshots establish native rendering and metadata behavior, not live company-provider acceptance. Stable publication and final archive contents must be verified separately.

## Latest-source recapture

On 2026-10-14 the optimized cfbcdf4 executable was built and all four native screenshots were replaced. These show the current source layout, not the older prepared screenshots. The executable still reports 0.1.16 while the requested manual target is 0.1.17; the captured version was never retouched. A fresh three-worker fixture run verified parallel overlap, passing tests and exact approval/merge gates before the Agent Room was photographed. [Capture and binary fingerprints](../raw/2026-10-14-current-native-captures.json) distinguish this evidence from published-release acceptance.

## v0.1.24: frozen source and downloaded evidence

Publish only the frozen tagged source, then compare downloaded manuals to that Git commit even if the worktree subsequently changes. v0.1.24 source aa939ba6fc111bac28331b389fb51199df0e7aab passed [every Release gate](https://github.com/neodamul/AgentConductor/actions/runs/38592812353); [verification receipt](../../docs/evidence/v0.1.24-release.json) records both archive/binary hashes, bilingual manuals, screenshots and all four native workflow cases. Normalize CRLF/LF for text equality only; raw hashes remain byte-exact. Retain cancelled duplicate main validation separately from the successful Release run. Local disk/cache or60s subprocess timeouts are recorded honestly and never substitute for passing native package gates.
