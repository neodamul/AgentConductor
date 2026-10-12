# Fresh project settings

DD-85 separates configuring a directory from executing against a committed baseline. Requiring Repository::open during Save caused the reported git rev-parse failure for ordinary new folders. A plain folder is now saved and canonicalized without Git; planning fails before slug creation when a repository or initial commit is missing. Worktree isolation remains optional, while current diff/integration still needs Git. Quota collection is independent of this boundary. Non-Git execution needs a separate design decision.
