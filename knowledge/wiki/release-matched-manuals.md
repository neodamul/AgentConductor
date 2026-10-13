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
