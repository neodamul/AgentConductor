# Ordinary project folders

A selected project folder needs no Git repository. AgentConductor copies eligible source files into bounded private history and runs the existing plan/task/verify/complete workflow there. Git is required internally; the original gets no .git. Shared execution is default and worktrees remain optional.

The baseline binds original file hashes and executable bits. Starting and applying refuse concurrent source edits. Final file application follows exact diff/test approval and retains a receipt plus backups. A receipt left by an interrupted application requires investigation before retry.

Snapshots exclude common build directories, .env files, local app artifacts and OS metadata. Aliases/hard links and snapshots above 10,000 files or 256 MiB total are refused. Files above 16 MiB are fingerprinted incrementally with a fixed 64 KiB buffer; the former per-file cap is removed. Existing Git projects keep their branch-merge flow.

Trace: R-80, DD-88, QAS-75. Evidence: tests/folder_workflow.rs, tests/workspace_project_folder.rs, tests/startup_folder_plan.rs.

Windows private paths plus retained Git refs can exceed MAX_PATH. Git invocations enable core.longpaths locally to each command; no global Git configuration is changed. Evidence: Windows CI 38196650126 folder_workflow failure.

Superseded unstarted snapshots and safe orphans are pruned. Completed runs retain Git objects and audit/test/diff/backup evidence while private working files are discarded. Unknown or active ownership prevents cleanup. Completed knowledge inspection uses the original applied files.

## Large assets

A regular asset may be as large as the remaining 256 MiB snapshot allowance. SHA-256 is updated from a bounded read buffer instead of loading each complete file into memory. The scanner checks file metadata against the aggregate limit before hashing; it repeats the byte limit during the read to catch growth. `Source::validate` keeps the same hash/size/executable fingerprint and rejects edits before application. Git remains internal to the private history copy; the selected original folder is not initialized.

Trace: R-152 → BG-107 → SF-139 → UC-142 → FR-137/NFR-41 → QAS-148 → DDR-17 → DD-162. Evidence: `tests/folder_workflow.rs` (`snapshots_stream_single_files_larger_than_sixteen_mib`).
