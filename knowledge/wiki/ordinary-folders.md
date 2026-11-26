# Ordinary project folders

A selected project folder needs no Git repository. AgentConductor copies eligible source files into bounded private history and runs the existing plan/task/verify/complete workflow there. Git is required internally; the original gets no .git. Shared execution is default and worktrees remain optional.

The baseline binds original file hashes and executable bits. Starting and applying refuse concurrent source edits. Final file application follows exact diff/test approval and retains a receipt plus backups. A receipt left by an interrupted application requires investigation before retry.

Snapshots exclude common build directories, .env files, local app artifacts and OS metadata. Aliases/hard links and snapshots above 10,000 files, 16 MiB/file or 256 MiB total are refused. Existing Git projects keep their branch-merge flow.

Trace: R-80, DD-88, QAS-75. Evidence: tests/folder_workflow.rs, tests/workspace_project_folder.rs, tests/startup_folder_plan.rs.

Windows private paths plus retained Git refs can exceed MAX_PATH. Git invocations enable core.longpaths locally to each command; no global Git configuration is changed. Evidence: Windows CI 38196650126 folder_workflow failure.

Superseded unstarted snapshots and safe orphans are pruned. Completed runs retain Git objects and audit/test/diff/backup evidence while private working files are discarded. Unknown or active ownership prevents cleanup. Completed knowledge inspection uses the original applied files.

## Match folder inventory and external Git exclusions

[GitHub #1101](https://github.com/neodamul/AgentConductor/issues/1101) reproduced a false dirty-baseline error at **Approve and run**. Folder planning already excluded case-insensitive environment/cache names and directories marked as Goals at any depth. The external Git history now uses exact exclusions from that same inventory, including when a previous activation left the history in place. The snapshot, clean check and approval digest still protect the actual source.
