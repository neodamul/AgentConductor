# Fresh project settings

DD-85 separates configuring a directory from executing against a committed baseline. Requiring Repository::open during Save caused the reported git rev-parse failure for ordinary new folders. A plain folder is now saved and canonicalized without Git; planning fails before slug creation when a repository or initial commit is missing. Worktree isolation remains optional, while current diff/integration still needs Git. Quota collection is independent of this boundary. Non-Git execution needs a separate design decision.

## Fresh coordinator default (#899)

Source: [fingerprinted native-dispatch regression](../raw/workspace-coordinator-default-899.json). The machine master list is adopted after an empty Workspace is constructed. Initialize the coordinator only after that adoption, only for fresh settings, from profiles.first() without sorting. Persist the adopted profiles/default once before exposing the Workspace. A saved valid non-first assignment survives reopen; existing unresolved assignments remain available for explicit reassignment. An empty team does not invent or launch an agent. Trace: R-101 / UC-91 / QAS-97 / DD-109.
