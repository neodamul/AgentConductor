# Changelog

Releases for macOS (Apple silicon) and Windows (x86_64). Stable versions start at
0.1.0 and continue with 0.1.1. Historical preview entries are retained. Each section is published as
the GitHub Release notes for its tag.

## Unreleased

- In English the agent card's tool row (Edit, Models, remove, up, down) stays on one line at both reference widths instead of wrapping its last button (#654).
- The agent card shows `Claude Code` (not `claude`) everywhere in the UI, keeps bottom space when it carries many tags, and shortens the alternate chip to `Alt → agent` only when it would wrap (#654).
- While the usage popup refreshes only the ↻ arrow spins, not the whole button with its hover background (#647).
- A fallback (or a manual reassignment) hands the old agent's conversation to the next agent: Claude Code and Codex sessions are read from their session stores and OpenCode's with `opencode session export`, flattened to `role: text` lines (tool calls, tool output and reasoning left out, 32 KiB of the newest turns) and added to the handoff as an untrusted record. The way to read a session is preset data (`invocation.session.export`); a profile turns it off with `"export": null` in its agent settings (#639).
- Resume says why it has nothing to do ("Nothing to resume: task X is failed" / "재개할 것 없음: 태스크 X가 failed") instead of silently doing nothing (#637).
- A coordinator review that ends `blocked` without revisions is retried by Resume, and the task notice offers **Retry** and **Reassign an agent** (#637).
- A refusal sentence printed by a built-in CLI (Claude Code, Codex, OpenCode) ends the task as "Permission denied", like a declared denial, instead of a plain failure (#637).
- Resume on a failed run now works: it retries the failed tasks (keeping their work and session), runs a failed integration verification again, and restarts the repair rounds, instead of failing again at once; an out-of-scope failure still points to the scope recovery dialog (#625).
- A built-in CLI (Claude Code, Codex, OpenCode) that prints a plain-text line instead of JSON — a notice or error page from a company gateway or wrapper — no longer fails the task with "invalid JSONL": the line is kept as a warning, the session id and result that follow are still read (so the task can be resumed), and a task that then fails quotes what the CLI printed (#624).
- The English "Stop run" button no longer shows a doubled stop glyph (#621).
- The Agent Room card keeps a usable area for the activity, diff, log and lesson tabs when the card is crowded (the review step with its summary); the card scrolls instead of squeezing the tab content to nothing (#619).
- The English Workspace page title is capitalised ("Workspace settings"), like the card below it (#617).
- Repairs two merge-conflict marker lines left in the design-parity spec and adds a test that fails when any tracked file contains one (#614).
- Test-suite fixes after the Room card work: the status dot's waiting colour is a theme token, the choice-controls test reads the task id from the card's meta line, and the design-parity checks no longer depend on 1–2 px font-metric differences between macOS and Windows (#609).
- The Agent Room's activity rows follow the approved design: a number (or range) column, a bold title, a soft detail line and a rule between rows; the "thinking" row says "출력 없이 추론 중입니다." (#607).
- After "New Goal" (no active Goal, empty text, other Goals exist) the Workspace step shows the design's "새 목표" state in its head and the footer note "작업 디렉터리·코디네이터는 유지하고 새 Goal을 추가합니다." (#600).
- The design-verification suite now compares the choice popup (agent/model/coordinator menus) with the approved assignment menu: box, heading and options (#605).
- The task card names why a task is stopped, like the approved design: a warning block when it waits for a tool decision (the requested command and who decides) and an error block when the CLI denied a tool (the denial and how many dependent tasks are waiting) (#599).
- Once the run waits for approval the Agent Room shows the approved design's review card: "6단계 · 검토·병합" (the stepper now marks step 6 from approval on), a summary line with the changed-file count, the integration test result and the coordinator's recommendation, and the changed files with their added and removed lines (#598).
- The Agent Room's verify step shows the approved design's card: "5단계 · 실행 · 검증", the coordinator's title and intro, and one row per task with its test command and result (passed / failed / running / waiting) above the coordinator's timeline (#597).
- The Agent Room's task view is one card under the step tabs, like the approved design: the head reads "4단계 · 실행 · 작업 · 선택한 작업", the task summary, and "agent · task id · first owned path", with the status tag on the right; the activity tabs, instruction box and Stop/Resume/Recover buttons sit inside the card (#580).
- The task-details panel shows the selected task's instruction, its dependencies and its test command under the ownership path, as in the approved design (#581).
- The Workspace footer's idle text is the design's "연결됨 · 로컬 실행" / "Connected · local run" (the folder and branch are named on its right side); progress and stale states keep their own texts (#583).
- The Claude usage lookup no longer inherits the Claude Code session that started the app (session markers such as CLAUDECODE and CLAUDE_CODE_ENTRYPOINT are dropped; login tokens, keys and provider settings are kept), so an app started from a Claude Code terminal reads the subscription usage instead of reporting API billing or a lost login (#585).
- The footer buttons of every step card (Workspace, Agent roles, Plan) are one size, the Workspace step's 34px with 8/13px padding, 6px corners and 13px text, as in the approved design; before, steps 2 and 3 had taller buttons of different heights (#579).
- The application menu popup follows the approved design: 8px corners, 230-310px width and 33px rows with 5px radius (its items stay the app's full inventory) (#563).
- The Goal list slide-over is 730px wide and the coordinator chat panel 460px with 24px padding, like the approved design's slide-over panels; Settings already matched (#563).
- Dialogs follow the approved design: 24px padding, 12px corners and 700 titles everywhere; the close-window and agent dialogs have the design's widths (620 / 560px) and the Goal result popup is 660px with 25px padding and 14px corners (#563).
- The Agent roles and Plan step cards use the approved design's card padding (20/24px) and 11.5px eyebrow; only the Workspace card keeps 17/22px (#563).
- Claude usage lookup: a `/usage` screen that cannot be read (a signed-out Claude CLI or one billed per API use) is reported as soon as it has settled instead of after three minutes, so the refresh icon stops spinning and the message says to sign in with /login (#568).

- The right task-details panel follows the approved design: 330px wide (docked as a third column down to 700px), page background, 22/18px padding, an 18px heading with a rule, a context line (agent · task · title, or a hint before a plan exists) and section titles in the design's wording with a rule under each ("소유 경로와 작업", "담당과 모델", "가용성과 재개 예약", "검토 후보", "계정 사용량") (#561).
- The footer shows a status dot on the left and, on the right, the Workspace folder and its Git branch with their icons (full path on hover; detached HEAD shows a short commit; nothing without a chosen folder), on the Workspace screen and in the Agent Room (#562).
- The step-tab groups name themselves: the icon of each group now has its "준비" / "실행" / "마무리" label to its right, like a normal button (#558).
- Step cards: only the card body scrolls; the Goal header, the stepper, the card header and the action footer stay in place (the plan no longer slides behind the header) (#543).
- The Codex logo no longer has a white tile, so it sits on its avatar like Claude's without a gray ring (#545).
- Jira items are called issues everywhere they are shown ("Jira 이슈" / "Jira issue": the import button, the reference dialog, the import error text and the documents), not stories.
- Agent cards show the dual-ring quota icon from the provider's windows; clicking it opens a popup with each window's remaining share, reset time, observation time and, on errors, the failure with a retry (#426).

- UI: every text and control size, weight, colour and padding now follows the approved mock (one strict size scale, no browser-default 16px controls); a typography parity set checks it in both themes (#432).
- Both manuals have a new section "The goal screen" (1.2a) describing the six-step stepper, the panels, the Goal list actions, the result popup and the status borders.
- The completed-goal popup shows its summary as one ruled line (changed files, tests passed, lessons) instead of three stacked tiles.
- The main action buttons of a step (save, generate plan, approve and run, read plan, write plan manually, discard changes) show a small icon in front of their label.
- A title bar button hides and restores the whole left panel on the Workspace screen and in the Agent Room; the centre uses the freed space.
- The Goal header is a compact single line (17px) without the small caption above it.
- The Activity, Changes and tests, Raw logs and Lessons tabs of a task are compact pill-shaped tabs that fit their labels instead of an underlined row.
- In every step card the title and the action buttons stay visible while only the middle of the card scrolls, so a short window no longer hides Save or Approve.
- The Prepare, Run and Finish groups of the stepper have their own subdued background colours (light and dark) and a small group icon, as in the approved design.
- Agent cards no longer print the executable path or a fixed session capacity, have 8px corners with a red removal icon, and task rows in the left panel are quiet rows with a trailing chevron, a hover fill and an accent edge on the selected one; the Agent Room's left panel is now really 250px wide when the task panel is closed.
- Each Goal in the Goal list has a red trash button (permanent deletion behind a centered confirmation naming the Goal and its exact folder; Cancel is the default), a folder button that opens the Goal's folder in Finder/Explorer, and — for a completed Goal — a **Result** button that opens the same completed-goal popup as the Agent Room.
- The left panel is 250px wide with equal side padding on the Workspace screen and in the Agent Room, its two tabs are 29px high, the six step tabs are a single 35px row and the version in the title bar is smaller, matching the approved design.
- The team briefing, the coordinator chat and Settings now open as right-hand slide-overs over the unchanged screen, like the Goal list; only one is open at a time (opening Settings or the briefing closes the chat).
- Claude Code and Codex now show their own color marks (two MIT-licensed Lobe Icons files embedded in the app and shipped with their license text) instead of the installed application's icon or letters; task avatars are 28px and agent avatars 29px with the status border. OpenCode keeps its installed-application icon.
- In the Agent Room the task status is shown by the border of the agent's icon (green running or completed, yellow waiting, red failed or blocked) instead of a separate dot; only a running task pulses. The icon is larger so the border reads.
- When a goal completes, one centered result popup shows where the changes were merged, the changed files, the verification results and the lessons. It opens once when a run completes in the Agent Room and from a new **Result** button in the left panel afterwards.
- The goals of a workspace open in a slide-over from an outlined **Goal list** button with a count badge, on the Workspace screen and in the Agent Room, instead of an inline card. The list and its actions are unchanged.
- The Agent Room's task details column is now docked: it starts closed every time a run is opened and is opened and closed only with the panel icon in the titlebar. It no longer remembers its state or opens by itself; when a decision is waiting, the icon shows a blinking dot.
- The left panel has two tabs on both screens, **Agents** and **Phases & tasks**. The agent cards (with Add, quota, Edit, Models, Remove, order and drag) are also available in the Agent Room; steps 1–2 open on Agents and later steps on Phases & tasks. The Workspace screen lists the plan's tasks under Phases & tasks.
- One stepper with six steps (Workspace, Agent roles, Plan, Tasks, Verify, Review and merge, grouped Prepare / Run / Finish) now serves both the Workspace settings and the Agent Room. Steps 4–6 open the Room while a run exists and the Room marks the step of the run; steps 1–3 take you back to the Workspace panels. The Room's separate five-stage strip is gone.
- Closing the window while a run is working now asks first: keep the run going in the background (the default), stop it and close, or cancel. A run keeps working after the app is closed and is shown again when the app reopens.
- The Agent Room folds a run of thinking updates into one Activity line ("Thinking… n updates") and the run banner says how often the run was interrupted and recovered, with the time and outcome of the last recovery.
- A custom CLI that refuses a tool (a matched `denial_condition`) now ends its task as **Permission denied** instead of Failed: tasks that depend on it wait, the run is blocked with the tool and the CLI's message, and Resume retries it after the CLI's configuration is changed.
- Engineering documents: the requirement, use case and quality-scenario documents are consolidated (a reused identifier chain renumbered, superseded and amended entries marked, issue fixes traced, the index rebuilt), a test keeps them consistent, and docs/14 records which requirements are verified by tests and which are not yet implemented.

- Step 2 (agent roles) has a Save button. Saving agent or role changes (for example after an agent was deleted) re-assigns the tasks of the Goal's reviewed plan to the agents the roles now allow instead of discarding the plan, so there is no need to generate a plan again (#564).
- The Workspace step puts prompt optimization, Jira, Confluence and document preview on one row, in that order (#560).

## v0.1.60
- **Messenger channel interface** (#515, phases #517-#519): Slack and the company webhook now sit behind one channel interface, so another messenger is one implementation. The notices are an enumerated catalogue with localized texts; at most ten a minute per workspace are sent (the rest are counted and reported once), and notices that carry an answer code are never limited. More than 20 pending tool requests are announced 20 at a time, the rest follow in order as answers free slots.
- **Talk to the coordinator from the messenger** (#519): listed talkers ask the read-only coordinator questions in the Slack channel or company relay and get the answer back (at most three messages). Talkers are separate from approvers, a message is only a question, the first poll skips earlier history, limits (2,000 characters, six messages per ten minutes, queue of five) are answered with a short reply, and it only reads while the desktop is open (30 s in a conversation, 2 min idle).
- Security: shared workspace files cannot activate messenger delivery or decisions, assign approvers or talkers, or choose a Slack reply channel on export or import (#523).
- Messenger dialog: the text beside each check box is left-aligned next to the box (#544).
- Popups no longer open with a blue focus ring on their first button or check box (the Messenger dialog, the Goal list panel, …); text fields keep the focus and Tab still reaches every control (#546).

## v0.1.59
- Creating a workspace is no longer blocked by unrelated work. The **Create** button in the New workspace dialog stays enabled while a quota refresh, model discovery or another action runs; it now waits for that work, says so in the dialog ("Another task is still running; the workspace is created as soon as it ends.") and creates the workspace when it ends. After any action the buttons are re-enabled before the next status refresh finishes (GitHub #511).
- The coordinator chat shows what the coordinator is doing: while it works, the single "working" bubble lists the newest tool calls and text of the CLI (Claude Code and Codex output, at most eight short lines) and the age of its last output ("last output 12s ago" or "no output yet"), so a stalled run is distinguishable from a slow one. The one-second timer updates only the status line, and a reader who scrolled up is not pulled down (GitHub #514).

## v0.1.58
- A custom CLI's output that is not JSON no longer ends as a bare "invalid JSONL": the failure quotes the line and shows the end of the CLI's stdout and stderr, labeled apart.
- A custom profile can set `on_invalid_json: "text"` to keep non-JSON lines as progress text (a result event and a zero exit are still required, and text that reads as a refusal fails with "check the CLI configuration").
- A custom profile can declare a `denial_condition` (event type, JSON pointer with `equals` or `contains`, tool and message pointers). A matching event fails the task as "permission denied: tool …: …" even when the CLI exits 0.
- An agent set to approve tools automatically but with nothing that could show a refusal gets a "Check configuration" badge on its card.

## v0.1.57
- Codex planning no longer fails with `invalid_json_schema: Missing 'task_type'`: the plan contract now lists every task field as required (a task states `task_type` and `execution_options.effort` as null when it has none), so Codex Structured Outputs accepts it.
- A failed planning step shows both what the CLI wrote to stderr and to stdout, so an unrelated warning (a skill that failed to load) can no longer hide the API's rejection.

## v0.1.56

- Korean interface: the reason a coordinator cannot chat (shown under the coordinator selector and in the chat) is now in Korean (#333).
- App update: when work blocks an update, the message now names each blocking workspace, kind of work and how long it has run, and says to wait or to stop it; a check that failed (state could not be read or a process could not be proven gone) is listed apart as "not running work". A run still marked "running" whose coordinator is gone no longer blocks the update (#331).

## v0.1.55

- Custom CLI coordinator: an agent can declare read-only arguments (agent form, JSON array); the coordinator chat then runs the CLI with them, so a custom CLI that plans can also answer chat. Without them the chat is still refused, and the workspace settings now warn under the coordinator selector with the reason as soon as such an agent is chosen (#326).
- Coordinator chat: the chat icon now works on the run screen too (its panel was hidden with the workspace screen), and an answer that is still pending is followed there (#325).
- Coordinator chat: a coordinator whose permission cannot be run read-only is refused with the agent's name and what to change (full access or native callback in the team settings, or another coordinator) instead of "coordinator requires read-only capabilities" (#325).
- Agent list: the coordinator's card says "Coordinator" and the list redraws when the workspace or the coordinator changes (#325).
- Workspace settings: the coordinator's model is chosen next to the coordinator, from that agent's queried models; "CLI default" keeps the previous behavior (#325).
- Integration failure: the failure banner and the drafted retry feedback show both stderr and stdout (labeled, stdout last, each limited to its last 8 lines with the skipped count), so a warning on stderr no longer hides the real error on stdout; a single stream is shown as before (#323).
- Agent cards no longer say "Querying usage" with a running clock; the refresh icon already shows that the query runs. The run screen still counts it (#321).

## v0.1.54

- Claude and Codex usage: a login that expired and a CLI never signed in are now told the same way ("not signed in or the login expired" with the command to sign in again) instead of ending as a timeout or a generic failure; a Codex API-key login keeps its own advice (#317).

## v0.1.53

- Claude usage: the query is given up after 3 minutes instead of 1, for PCs where it takes longer than a minute; a fast PC is unaffected because the query ends as soon as the usage is read (#315).
- Claude usage: a company allocation written with thousands separators (`$1,351.23 / $5,000.00 spent`) and a terminal that draws spaces as cursor moves are now read, and a usage screen that was read but not understood is named in the error (`screen not understood: ...`) instead of ending as a bare timeout (#295).

## v0.1.52

- Docs and release: tests now fail when the README or a manual sentence does not name the package version, when a screenshot they show is missing or a captured screen is not shown, and the release workflow refuses to publish when the interface changed after the screens were captured, unless a commit confirms they still match (#290).
- Claude usage: the query is given up after 60 s instead of 20 s, so a slow company PC (scanning, managed settings, proxy) can still report usage, and a timeout names the limit and the stage. While a usage query runs, the agent card and the run screen show how long it has run, and the run screen's refresh button is marked busy instead of only disabled (#297).
- Tests: every kind of request the JS pages send must be classified (job with a progress rendering, refresh, or direct), so a request whose wait shows nothing cannot be added unnoticed (#301).

## v0.1.51

- Workspace and run screen: every button that asks the app for something shows that it is working until the answer, and after a second the status line (the connection line in the run screen) names the request with its time, for example while a workspace is deleted, the messenger is tested or the team briefing is read. Before, about 30 of these showed nothing until the answer or a timeout. An idle page no longer risks saying for a moment that the app sent no new state (#285).

## v0.1.50

- Workspace: the coordinator chat, the goal rewrite, the MCP lookup (review window, tool discovery and source line) and the app update check and install now show how long they have run, and say when the app has sent no new state for a few seconds. A chat that is still running when the workspace reopens counts from its start instead of from the reopening. Typing a `jira:` or `confluence:` reference enables the source preview at once instead of after the next refresh, up to 5 s later (#279).

## v0.1.49

- Docs: the user manual's header said 0.1.30; it now names the released version and a test keeps it equal to the package version. The README and manual screenshots are recaptured from the current interface (they showed v0.1.21).
- Workspace: confirming "Approve plan and run" did not send the start. Since v0.1.30 the start feedback looked up a wrong element, so the handler stopped with an error (`null is not an object`) before sending. From then on every refresh of the page failed the same way and the screen stopped updating. The start is sent again, and its line counts the time until the run report exists. The line also stays after the workspace reopens, and a start that fails keeps its cause under the run button instead of only a short toast. "Back to run" is available while the run executes.
- Workspace: planning and starting progress shows under the step tabs on every tab, and a click opens Review the plan. Elapsed times keep counting every second from the recorded start, also when the app is slow to answer. After 5 s without a new state the line and the footer say how long nothing came. A goal rewrite shows its elapsed time and is followed again after reopening. A model query in progress or queued is shown on the agent card.
- Run screen: a pending task says what it waits for (a dependency, the automatic retry time, a free session or the account). A running task's row shows how long ago it last wrote output. After the last task the banner and the integration row name the test in progress, its time and its last output, instead of "agents are working". The banner says when a stop is in progress and when the run's process is gone while the run is still marked running (Recover continues it). Actions show their elapsed time, and the banner and footer say how long no new state came.
- Tests: every `tests/*.cjs` UI test now runs in CI (13 had no runner), and `tests/ui_sim` holds a simulator of the native bridge to check what each step shows.

## v0.1.48

- Phases: after a phase finished, the workspace could keep showing "starting agent execution" with every button disabled (no review of the next phase) until the app was restarted. The mark that says a start is in progress was not ended when the run opened after its report appeared late, nor when a start job ended without opening a run; it now ends in both cases.

- Run screen: the coordinator Activity tab starts with a status board (run state, completed / running / failed / pending counts and one row per task with its agent, status and last output) and no longer lists the `workflow` entries, instead of repeating `AgentConductor run=… phase=running completed=0 failed=0` once per report save. The recorded history is unchanged.

## v0.1.47

- Chat: the coordinator can now be asked while a plan is being made or a run is starting (it had its own refusal "startup operation still running", and the message was rolled back with only a toast). The chat runs in its own slot, is told which job runs, for how long and how much output its process has written and when it last wrote, so it can answer why a plan takes long, and a question that cannot be accepted is shown in the conversation. The snapshot gains `chat_job`; the chat is no longer reported as `job`.
- Plan review: the line with a task's type and effort is no longer squeezed into the one-character-wide arrow column (it ran down the page as a tall column of single letters); it now sits under the task description at full width.

## v0.1.46

- Requests: the periodic state request, quota refresh and chat no longer queue behind a slow Git call. The state revision reads the repository HEAD from the Git files instead of starting `git` (a company PC where each `git` call took 2.5 s needed over 15 s per snapshot and made the chat time out), a layout the files cannot answer is refreshed by a short background task, and the snapshot answer is built after the settings lock is released. The request diagnostics log now also names a wait for the lock or a step of half a second or more (`step name=lock|advance|snapshot ms=...`).

## v0.1.45

- Planning: a task without an owned path (a verification task too) is now part of the planning contract: the output schema and the prompt require at least one path and a test command, a rejected task is reported with its id and the wrong field instead of one combined message, and an empty scope gets the same single correction round as an unsafe or overlapping one.

## v0.1.44

- Coordinator replies: a complete fenced JSON document is now found even when an unfinished fenced draft comes before it (the second opening marker was taken for the first block's closing one, so planning failed). The coordinator reply limit is raised from 64 KiB to 256 KiB.

## v0.1.43

- Chat: every question now carries a Workspace section with the goal, the latest run's tasks, test results, changed files and candidate diff (cut when long). The read-only coordinator can read the project folder but not the run folder, so before this it could not say what was built or how to run it (#243).
- Chat: when a send times out and the stored conversation cannot be read either, the message stays in the conversation as pending instead of being put back into the input. Once the conversation can be read, an accepted message is followed to its answer with the input still empty; a message the application never received returns to the input with the error. Before, the same text showed in the input and in the conversation when the late answer arrived (#241).

## v0.1.42

- Every prompt sent to an agent now has the same layout (Document, Task, Format): the reviewer and worker prompts join the chat, planning and optimizer prompts, so an agent CLI that cannot be changed still receives a predictable prompt. A coordinator reply that is not the requested JSON is asked for once more, with the reason, before the call fails; the error now shows how the reply began (GitHub #235).

## v0.1.41

- The coordinator chat button in the title bar shows a chat-bubble robot icon instead of a "?" (GitHub #229).
- A background refresh that misses the application's reply is now retried quietly and the "Request timed out" toast appears only after three misses in a row, so an idle window no longer flashes it; every timeout the page gives up on is also written to the diagnostics log with how long the page really waited and whether it was hidden, and the manual explains how to read it (GitHub #221).
- The app now opens the workspace you used last. A workspace with no name, goal, project folder, plan or run is never listed; with none left the app asks you to add a first workspace, so a deleted default workspace no longer comes back after a restart or an update (GitHub #226).

## v0.1.40

- Idle workspace and Agent Room polling no longer raise a timeout toast on their own; request deadlines pause while the page is hidden and resume after it returns (GitHub #221).

- `agentconductor io conformance <executable>` runs a CLI through the acio contract (self-description, a read-only question with a valid event stream, rejection of an invalid input) and prints a pass/fail table; both manuals gained an "Integrating a CLI" chapter (GitHub #212).

- The prompts for the coordinator chat, Optimize prompt and plan generation are now assembled as Document (the material), Task (what to do and the current question) and Format (the shape of the answer), with the wording unchanged (GitHub #211).

- Each plan task now has a kind chosen by the coordinator (classify or extract, summarize, analyze, code, long agent work, creative). The effort a task runs with follows your choice, then the agent's default for that kind, and is lowered with a warning when the model does not honor it; the review shows the kind and effort of every task and the roles table shows the efforts a model honors (GitHub #210).

- What differs between the built-in agent CLIs (arguments, resume, model flags, full access, folder rules, capabilities) is now shipped preset data instead of code, and a profile can override session flags with `agent_settings`; the argument lists are unchanged (GitHub #209).

- Agent profiles can set `protocol: acio` on a custom CLI: its events need no mapping tables, tool requests that need a decision wait for one (denied when none comes), the supervisor can send a user message and a cancel line while it runs, and `agentconductor io describe <exe>` registers a CLI from its `--acio-describe` document (GitHub #208).

- `agentconductor io schema` prints the acio CLI I/O schemas (input, event, describe, control, agent settings) and `io validate` checks a document or a captured event stream against them, including the stream rules (GitHub #207).

## v0.1.39

- Request diagnostics: the "Request timed out" notice names the request kind, and the application records every chat request and every request of one second or more in `~/.agentconductor/request-diagnostics.log` (kind, size, duration, failure reason; no content). The manuals explain how to read it (GitHub #200).
- Coordinator chat: a request that timed out no longer removes your question or blocks the next one; the screen checks what the application received, keeps an accepted question, follows the running coordinator and shows its late answer (GitHub #197).

## v0.1.38

- **Agent tools guide…** and **User manual** moved from the workspace menu into Settings, beside MCP servers and Clear cache.
- Ask the coordinator now keeps the conversation with the workspace and shows it again after reopening, as a terminal-style transcript (Enter sends, Shift+Enter breaks the line, Up recalls earlier questions, New conversation starts over). A long earlier answer no longer blocks the next question. The chat still runs one read-only coordinator call per question; it is not a long-lived CLI session.
- Removed fixed texts that carried no information: the title bar "local · existing CLI login" label, the sidebar tagline under the agent list and the footer "shared by default · worktree optional" line.
- Optimize prompt: a button below the goal field asks the coordinator to rewrite the goal with the bundled Sentry `prompt-optimizer` skill (Apache-2.0, license shipped in the package); it reads the project read-only and shows the proposed goal, the changes and open questions, and the goal only changes when you choose Use this goal (GitHub #180).
- Integration tests no longer call `env::set_var` / `remove_var` directly: a shared helper serializes and restores environment changes, every remaining test `unsafe` call states why it is sound, and the CI safety-comment gate now covers test targets (GitHub #176).
- Every `unsafe` block in the application now states why it is sound, and CI on macOS and Windows rejects a new one without a `// SAFETY:` comment; the icon code is split into per-platform files (GitHub #172).
- A short Rust review policy is documented in `docs/13-rust-review-policy.md`: sixteen rules adopted from the Google Rust Style Guide for new and changed code, with the excluded rules, their reasons and a measured baseline; no existing code or check changes (GitHub #169).

## v0.1.37

- Credit balances and allocations are shown as whole numbers instead of long decimals such as 6667.28056954845 over 12500; the stored values keep their precision (GitHub #162).
- Feedback can be sent to the coordinator: select it in the execution list and the feedback box names it as the recipient, keeps its own draft and sends the request to the coordinator workflow instead of a development task; the coordinator investigates with the run evidence, answers in the judgment list with the request shown as queued, processing, answered or failed, and revises tasks only when the evidence requires it, so finished tasks are not repeated and a request no longer fails when the room is polling the run at the moment the coordinator starts (GitHub #165).
- Each run records a charter, the goal and every task's original paths, test command and dependencies, that the coordinator receives with every review; amendments may add at most eight paths in total beyond it (more stops the run with goal_drift), each applied amendment shows its drift, and a scope change the operator approves replaces the charter (GitHub #149, step 4: the coordinator guardrail is complete).
- The coordinator can amend the plan within limits: when a task leaves its owned paths it adds the exact file or nested path or a task dependency as a typed amendment that is validated against the plan rules, limited to five per run, listed with the coordinator judgments with an Undo amendment button, and the run continues; whole top-level directories, secrets or key paths, refused changes and the sixth amendment stop the run with the reason for a person (GitHub #149, step 3 of the coordinator guardrail).
- A launch gate refuses to start a run that cannot work: every agent and every task test command must resolve on PATH, as an absolute path or inside the project, and every problem is listed by code and task before any run folder exists; a script a task will write inside its own paths is allowed; a passed gate is listed with the coordinator judgments (GitHub #149, step 2 of the coordinator guardrail).
- Every judgment the coordinator makes is listed and cannot be hidden: tool decisions, review verdicts, replaced test commands, automatic time-out resumes, recoveries, hand-offs and lessons appear in the coordinator view and in the activity of their task, and the coordinator row shows the count (GitHub #149, first step of the coordinator guardrail).
- The Agent Room task details column on the right folds with a toggle and remembers the choice; it opens by itself when the run needs a decision with the candidate review on top and a blinking warning dot on the folded strip reminds the operator until the decision is made (GitHub #146).
- Merging or rejecting a worktree-mode run now removes its linked Git worktrees from the repository, and deleting a workspace or clearing its cache releases them first; the diff and logs stay in the run folder (GitHub #143).
- CI tests for cooperative workers and process containment now wait for observable readiness instead of fixed delays (GitHub #156).

## v0.1.36

- Lessons learned at each step are shown and shared: a coordinator revision, a replaced test command and an approved scope change are recorded as step lessons, listed under Lessons in the Agent Room, added to every later task prompt as evidence, and queued for agentmemory (GitHub #138).
- The coordinator now decides when a task fails, exits without a result or stalls until it cannot continue: it reviews the retained output, revises the task and the run resumes, within the same three review rounds as integration repair. Scope escapes, quota waits and approvals still ask a person (GitHub #135).

## v0.1.35

- Settings has an **Update repository** field: a GitHub repository name or an https URL of any GitHub-compatible host such as an in-house GitHub Enterprise server; release notes and assets come from its latest release (GitHub #130).
- The available-update icon is a downward arrow, the conventional download symbol (GitHub #127).

## v0.1.34

- Guided recovery from an out-of-scope failure: **Review recovery plan** shows every offending file, each task's approved paths and the files moved to stray-files, previews and applies an approved change to owned paths and the test command, restores moved files without overwriting, and resumes from the finished work (GitHub #122).

## v0.1.33

- The workspace screen has a read-only chat with the coordinator (the ? button in the title bar): ask how to run or use what was built, where results are, or how to use the screen; it answers from the project, its plans and the manual and gives commands for you to run (GitHub #115).
- In the Agent Room, the selection follows the run: when the session you watch finishes it moves to the next running session, or to the coordinator entry when the run waits for approval (GitHub #109).
- The Agent Room activity line keeps a fixed one-line slot, so the tabs below no longer move while it updates (GitHub #117).

## v0.1.32

- In the Agent Room, a task that has not started can be given another agent or model while other tasks are running; the change is queued and applied before the task launches (GitHub #103).
- The Agent Room shows, for a running task, the tool in progress with its elapsed time, the time since the last output and the agent process state, and pulses the running agent icon unless reduced motion is set; silence is never labelled thinking or failure (GitHub #100).
- The run screen's Send feedback is enabled for a running task, so the queued active-task feedback is reachable from the UI (GitHub #95).
- Quota reset times now resolve for about seventy common IANA zones, including daylight saving (US, EU, Australia, New Zealand); zones outside the table still show no reset time (GitHub #82).
- A finished update now removes its downloaded archive and unpacked package, and stale folders of earlier releases (GitHub #80).
- Remote MCP server headers accept the documented `Bearer {env:NAME}` and `Basic {env:NAME}` forms; literal secrets stay rejected (GitHub #84).
- Task file scopes are compared without regard to case, so `README.md` and `readme.md` can no longer be owned by two tasks on macOS and Windows (GitHub #79).
- Shell commands whose program is assembled by the shell (glued quotes, variable expansion) or given as inline interpreter code (`sh -c`, `python -c`, `node -e`, `python -m http.server`, …) now ask a human instead of running at once (GitHub #83).
- Messenger approvals now require an explicit command: negated or conversational replies such as `don't allow CODE` are ignored (GitHub #81).
- Messenger alerts: a run that fails or waits for approval again after a retry now notifies again; only repeated saves of the same state stay silent (GitHub #72).

## v0.1.31

- Verify → Merge → Workspace finalization progress is shown in the Agent Room, with a visible busy primary action while an approved candidate merges (GitHub #50, #51).
- A generated phase whose tasks claim overlapping file scopes is re-planned once with the exact validation feedback instead of ending planning (GitHub #52, #53).
- Custom CLI profiles can also declare model catalogs, execution options and output handling, with clearer custom result errors (GitHub #62).
- SQLite writer contention is waited through instead of failing a parallel Windows workflow (GitHub #56).
- Native running-status indicators honor reduced-motion settings (GitHub #60).
- Reading a run no longer changes its database: `workflow::load` opens `state.sqlite` read-only, so a page or test polling a run that is just starting can no longer make it fail with "database is locked" (GitHub #65).
- Release notes are now extracted by the Rust CLI, keeping release publishing independent of a Python runtime.
- The offline English and Korean manual generator now runs in Rust; its superseded Python renderer and duplicate tests were removed.
- macOS Finder bundle assembly now uses the native Rust CLI; the duplicate Python bundle generator was removed.
- Release ZIP creation now runs through the Rust CLI, preserving file contents, executable metadata and bounded ZIP timestamps while refusing symlinks.
- Release-package manifest and SHA-256 verification now runs in Rust before a packaged binary is executed.
- Packaged release smoke checks now run through the Rust CLI, covering parallel scoped work, tests, diff review and approval-gated merge in Git and plain-folder workspaces.
- Native release package assembly now runs through the Rust CLI, including reviewed dependency licenses, notices, SBOM, checksums and archives.
- One project workspace holds multiple selectable goals: New goal preserves project/team and prior evidence while clearing old approvals and sources; prior goal runs remain reviewable. Duplicate folder binding is refused without losing inputs (GitHub #6).

## v0.1.30

- Custom CLI profiles can map JSONL events and session IDs and optionally expose a bounded quota command using the normalized Snapshot format (GitHub #12).
- Repository planning rules are loaded recursively, and timeout recovery and active-task feedback are more reliable (GitHub #8, #16, #19, #44).
- Company builds can show a validated local titlebar suffix; public release packages reject suffixed builds so update version checks remain numeric (GitHub #13).
- Release remains a stable macOS Apple silicon and Windows x86_64 package with bilingual offline manuals and verified checksums.

## v0.1.24

- Claude metadata recognizes shell and chevron prompts plus its known trust choice; explicit Enterprise spent/allocated output produces remaining capacity. Timeout errors identify the stage and date-only resets are not guessed (GitHub #5).

- Create/open/switch workspaces during active planning, metadata and runs; retain owned sessions and reconnect their current progress when returning. Deletion protection remains, and delayed replies from a previous workspace cannot overwrite the new view.

- Completed Agent Rooms no longer block new workspace creation or switching; active runs and native jobs remain protected (GitHub #4).

## v0.1.23

- Saving or changing the project folder resets Claude usage refresh immediately; automatic Claude queries wait until a project folder is saved.
- Planning specifies native CLI JSON Schema output contracts, keeping phases/tasks as arrays and retaining strict review and approval validation.

- Updated English/Korean Markdown and offline HTML manuals describe project-aware Claude usage refresh and schema-constrained planning; both formats ship in the native packages.

## v0.1.22

- English and Korean offline manuals use generated HTML with readable tables and embedded screenshots, loaded only when opened. Documentation and screenshots match this version.
- Windows desktop validation targets the window owned by its launched process and reports visible controls when startup fails.

## v0.1.21

- Export workspace and Import workspace are adjacent in the menu, with consistent English/Korean workspace labels.

- A model that replies it is at capacity is retried after a short wait, up to three times, for tasks and the coordinator.
- Files an agent creates outside its owned paths (for example a CLI's .aipro/plans) are moved out of the project into the run folder.
- Agent Room task rows show the reviewed task summary with the task ID as a label.
- A pressed button shows that its work is running until the request and any background job it started end, so it no longer looks disabled.
- The workspace page and the Agent Room stay open side by side: switch freely during a run; the room keeps its live progress and the workspace's back-to-run button shows the run state.

- Planner and worker instructions now follow the configured signal backend; Local Rust does not require agentmemory tools, and optional installation guides no longer imply a communication prerequisite.
- Windows folder/file/executable selection uses the system Common Item Dialog directly, removing per-click PowerShell, Windows Forms and C# compilation.

- Workspace step 1: the planning coordinator sits beside the workspace name and the project folder above the goal. MCP lookups show progress, Preview opens its window at once and fills it when the lookup ends, and jira:/confluence: references in the goal are tags with a hover summary that delete like text.
- Agent Room: account usage shows Enterprise budgets as a remaining percentage like the agent cards, and the task shown on the right is outlined in the list.

## v0.1.20
Includes every change prepared for v0.1.19, which was not published (its first release build failed).

- Codex reconnect attempts and explicit WebSocket-to-HTTP fallback notices no longer abort recovered sessions; final failures remain errors.
- The shared workspace menu opens an offline English/Korean user manual, including an Enterprise HTTP-only Codex configuration guide.
- Reviewing a finished run no longer takes the execution lease from the approving child.
- The goal heading's status follows the started run (running, waiting for approval, completed …) instead of showing "Started" forever.

- MCP documents: the registered coordinator selects read tools from enabled servers and resolves required metadata; server settings and catalogs stay linked without manual document-tool bindings.
- Large Jira/Confluence sources are preserved as full immutable local Markdown before coordinator summaries enter the goal context; original, Markdown and summary hashes are checked on approval and planning.
- Plan generation immediately clears previous failure messages, selects Review, and pulses the actual coordinator CLI icon without status colors. CLI cards no longer offer the paid-fallback tag/shortcut.
- Reverted plan/workspace edits no longer leave approval disabled. Actual unsaved edits and background jobs remain guarded with clear reasons.
- Claude api_error usage_limit_reached / HTTP 429 is parsed as an availability restriction even without type/is_error, retaining explicit reset times and the durable five-hour reviewed fallback policy. Duration telemetry never becomes a reset time.

- Custom agent CLIs (for example aipro set to auto-approve tools) run after plan approval as approving their own tools, like the full-access default, instead of being refused as unverified; no extra setting is needed.

- The update dialog shows the actual GitHub release notes beneath the release link in a keyboard-focusable, scrollable text panel.

## v0.1.18

- Compact account cards show percentages and a smaller refresh icon; reset times remain in accessible tooltips. Workspace steps have concise introductions and helper text uses the available width.
- Settings labels and controls have readable spacing; Local Rust hides the retained agentmemory server port. Agent tools guide is now in the workspace menu.
- Primary buttons and notification text remain readable in both themes, including disabled primary actions.
- Updated English/Korean manuals and real native screenshots.

- The custom CLI approval checkbox retains its normal inline size instead of inheriting full-width input dimensions.
- Custom agent CLIs that approve their own tools (for example one set to auto-approve tools) can be marked so in their agent settings and then run after plan approval instead of being refused as unverified.
- Folders without Git: the planning coordinator is told the project has no Git and plans no git test commands, git test commands still run against the history AgentConductor keeps outside the folder, and the coordinator review can replace a test command that cannot work (for example when an agent reports it) instead of deferring the failure; each replacement is recorded.
- **Themes**: choose Light (default; system-settings look with a blue accent) or Dark (the sage brand on dark surfaces) in ⚙ Settings → Theme. Information and layout are identical; the choice is remembered.
- Agent Room session rows are compact: the task name first, then a small agent icon, the agent name and a small status tag on one line; long or translated statuses wrap instead of leaving the session list.
- New `docs/design-guide.md`: color tokens, rules and checks for future UI changes.

## v0.1.17

- Windows scheduled CLI commands return promptly while their independent availability coordinator waits in the background.
- Idle availability coordinators no longer lock out CLI reassignment or model changes while waiting for a future reset.

- Agent Room task rows are compact: a smaller agent icon and the agent name on the left, the status tag at the right end of the same line, and the task name below (wrapping when long).
- A failed planning step shows why: Review the plan opens the error details (the coordinator's exit status and its own error output or error event) instead of only the log folder.
- Account reset times prefill availability with a countdown. Durable coordinator reservations resume due work across app closure/restart, cancel on reassignment or Stop, and use only a reviewed fallback when waiting exceeds five hours.

- Large diffs show a shortened preview and full-patch CLI instruction while allowing exact candidate approval. Evidence verification hashes the complete patch, including omitted content.

- Opening a started direct folder execution now checks its reviewed source binding, so it reconnects to the existing run instead of rejecting the planning copy / project path difference.
- Automatic updates: the app checks GitHub releases every 10 minutes; a newer version shows an update icon left of Settings. The icon or Check for updates in the workspace menu opens the update dialog, and Update and restart downloads the archive for this platform, verifies its published SHA-256, replaces the app and restarts it. Updating waits until no run is active. The CLI command `agentconductor update` installs the latest release without restarting.

## v0.1.16
Includes every change prepared for v0.1.15, which was not published.

- Workspace setup steps 1, 2 and 3 are tabs. Switching preserves drafts, supports keyboard navigation and opens plan review when planning starts or a plan is loaded.
- The startup page no longer stops loading partway (a leftover handler for the removed delete-dialog name field), so Escape and outside clicks close the workspace menu again.
- Agent CLIs run with full access by default: Claude without permission prompts and with the operator's own MCP servers (for example Playwright, so a verifier can open the web UI), Codex without approvals or sandbox. Only the standard claude and codex executables get it; the coordinator still plans and reviews read-only.
- When every task completed but integration verification failed (a failing test or a report recording a blocking issue), the coordinator reviews the evidence, revises the tasks that must change and the run resumes, for up to three rounds; Resume on such a failed run starts it. Findings the run cannot resolve are recorded as deferred follow-ups.
- Interrupted runs are recovered and resumed automatically when a workspace or the Agent Room opens (on by default; at most two retries without progress).
- A phased run records its goal phase when it starts, so a run whose process ended early still completes its phase in Goal and progress.
- The workspace menu closes when you click anywhere outside it (as with Escape).
- Open existing workspace lists workspaces only (no Delete button breaking the rows); a finished plan whose private folder copy was removed no longer blocks deleting or opening its workspace.
- The workspace list shows only workspaces the app manages, so every listed one can be opened and deleted; folders left by the Windows desktop test no longer appear (the test now uses its own HOME).
- Deleting a workspace no longer asks for a new workspace name: the most recently used other workspace opens, so several can be deleted in a row; deleting the last one leaves one empty workspace.
- Review the plan keeps each phase's tasks inside that phase; a plan whose phase already completed is no longer shown as under review.
- New ordinary-folder executions honor the selected project directory: shared task files and results are created there instead of in home-directory folder snapshots. Planning remains read-only; baseline commits and all Git metadata remain outside the project. Accept finalizes the result; Reject restores owned files with conflict checks. Existing runs and pre-existing Git metadata are preserved.

- Jira/Confluence document preview: discovered tools can be connected through Use for Jira / Use for Confluence with editable document arguments. Missing document-reader bindings now open configuration with clear guidance instead of making a registered MCP server appear missing.
- Interface wording is managed by stable message IDs in separate English/Korean JSON catalogs, including native app errors, menus and picker captions. Language changes preserve typed values, raw evidence and icons; missing translations fall back to English.
- Windows: Claude usage loads when Claude asks "Enter y/n:" to trust the project folder (screen-reader form); the query now answers y instead of Enter, which left it waiting until it timed out, and stops with guidance if Claude keeps asking.
- Agent Room status: a run status banner shows the state in bold colour, task and phase progress, why a run failed (the failed task and its test output) and the next step with its action (Review changes → Approve → Apply changes → Back to workspace); task states are bold and coloured, failed tasks are red and tasks waiting for a decision blink in the task list, which no longer shows usage badges.
- The coordinator is listed after the tasks as its own entry (integration verification); selecting it shows its run steps, tool decisions and integration test output. The interface uses the single name coordinator.
- Fix with feedback on a failed run selects the failed task and drafts a direction with the failure output.
- A verification report changed in the run that records a blocking issue (a Blocking/차단 section) now fails integration verification for the task responsible, instead of letting the run complete.
- Phased goals: Review the plan shows every phase, which one is under review and Plan phase n for the next one; a finished run says which phase completed and which is next. A completed phase is recorded even after its goal folder moved.
- Goal folders are created directly in the project folder as <project>/<goal>/ (for example C:\Temp\ladder\ladder_and_priority_games) instead of <project>/.agentconductor/<goal>/; existing ones move there when first used.
- Tool requests: the coordinator decides routine requests outside the workspace sandbox (also for runs started before v0.1.13) and each decision is listed in Activity with its reason; commands in a workspace stored under ~/.agentconductor no longer ask for their own folder. Allow once and Deny work at once in the request window while the run is working.
- Activity shows agent output like the CLI (messages, tool lines, result summaries); raw JSON stays in Raw logs.
- Agent CLI list: kept as one list for all workspaces (~/.agentconductor/agents.json) and not removed with a workspace; assignments to agents that are no longer listed are red and block Generate plan until reassigned; cards reorder by drag and drop or Move up/Move down, and every agent selection follows that order.
- Paths are shown without the Windows \\?\ prefix everywhere in the app and in messenger notifications.

## v0.1.14

Includes every v0.1.13 change below; v0.1.13 was not published because its macOS check failed.

- The macOS approval tests now match the workspace sandbox policy introduced in v0.1.13; leftover translations of the removed permission menu are gone.
- Generate plan shows "Preparing the project folder for planning…" with elapsed time as soon as it is pressed, instead of only "Workspace saved locally." while the folder is copied before the coordinator starts.
- Saving Workspace settings updates the workspace name in the titlebar at once; before, a renamed workspace (for example one created as "New workspace") kept showing its old name until the workspace menu was opened.
- Deleting the current workspace asks for the name of the fresh workspace that replaces it, instead of always naming it "New workspace".
- Claude usage loads again: the usage query accepts Claude's "Do you trust the files in this folder?" prompt (the query session has no tools, hooks or project settings) instead of stopping with an approval error; it stops with guidance if Claude keeps asking.
- The permission capability menu is removed from agent profiles; it follows from the CLI (Codex runs in its workspace sandbox, Claude behind the workspace policy). Existing workspaces are updated when opened.
- Claude tasks no longer stop for routine approvals: commands and searches confined to the workspace (builds, tests, `git status`/`diff`/`log`, file listings) run at once. Requests reaching outside the workspace (network, package installs, commit/push, privilege, outside paths, secrets) are decided by the workspace coordinator as orchestrator; a person is asked only when it cannot decide. A request the policy refuses denies that call only instead of ending the task.
- Time spent waiting for a tool decision no longer counts toward the 30-minute task limit, and a task that times out is blocked and continues on Resume instead of failing.

## v0.1.12

- Windows: Claude tasks can read and edit workspace files again instead of stopping with "control channel failed; owned process group terminated: tool path must be an unambiguous bounded workspace-relative path"; `C:\…`, `C:/…` and `\\?\C:\…` paths inside the workspace (any letter case, `\` or `/`) are checked by the same scope, traversal and protected-folder rules.
- Windows: an approved tool request no longer fails with "tool context verifier failed" while agents are busy; the pre-execution check of the approval allows 30 s instead of 10 s (it starts about a dozen Git processes, which took up to 6 s on an idle Windows machine).
- Tool requests read in plain terms: the action (Run a command, Edit a file, Read a file, …) with labelled inputs such as the command and its purpose, the file, or the text replaced and its replacement; the exact JSON stays under Raw input.
- Approving a tool request while the run is not running continues the run once the approval is recorded, and the approval window's button reads Close after a decision.
- The Agent Room uses the Workspace titlebar: the workspace menu and Settings stay available while a run is open (instead of the run's project path), and ← Workspace returns to the workspace screen.

## v0.1.11

- Preserve one external signal attempt across restart; uncertain delivery retains the local directive without replay, and a kernel-owned lease prevents concurrent publications.
- Settings → Signal transport chooses Local Rust (default) or agentmemory preferred for cooperative directives; when agentmemory is unavailable the same directive is delivered locally, and Settings and the briefing show the effective route and the reason.
- Windows: Generate plan, tasks and model discovery start npm-installed agents again instead of failing with "spawn agent: program not found" (`claude`) or os error 193 (`...\npm\codex`); bare names and npm shell shims resolve to their `.cmd` launcher.
- Windows: Approve plan & run starts Codex `native_sandbox` tasks whose executable is a full path such as `...\npm\codex` instead of failing with "native_sandbox requires the verified Codex initial workspace-write invocation"; the Codex read-only coordinator and Claude callback checks accept such paths too, and profile checks recognize `cmd.exe` and `powershell.exe` as shells.
- Two agents can be each other's fallback (for example Claude → Codex and Codex → Claude) instead of being rejected as a "cyclic fallback policy"; when every account on the chain is limited, the work waits for availability.
- Generate plan for a goal that already has a phased plan loads its current phase for review with the current agent profiles instead of failing with "this slug already has a plan".
- Agent Room session cards no longer wrap the task name one letter per line beside the quota badge: the quota sits above the agent name right of the icon, task and status use the full width below it, and the icon shows the status as a coloured border that pulses while running.
- A tool request waiting for your decision opens a window with the request, the time left before the worker stops and the task's live activity log; Allow once and Deny decide it there, Decide later keeps it in Tool requests, and the log keeps updating whatever is decided.
- Feedback sent to a run that is not running continues the run after the revision is recorded, so the agent answers instead of the run staying idle.

## v0.1.10

- Generate plan accepts phased plans whose first task depends on a task of an earlier phase instead of failing with "unknown dependency"; phases already run in order. An unknown dependency error now names the task and the missing ID.
- Generate plan accepts directory scopes written with a trailing slash (for example `docs/`) instead of failing with "unsafe or overlapping scope".
- Show the running build version beside the app title in Workspace and Agent Room.
- Bound completed disposable metadata cache at idle startup and after refreshes; add Settings -> Clear cache with a reviewed permanent-deletion preview.
- Stop a blocked Generate plan operation before deleting its current workspace; open a fresh workspace in the same window and preserve project sources and recoverable prior app data.

## v0.1.9

- Generate plan accepts a coordinator plan wrapped in a fenced ```json block instead of failing with "coordinator response contains no valid JSON tasks document".
- Windows: the C runtime is linked statically, so the app starts without the Visual C++ Redistributable (VCRUNTIME140.dll).

## v0.1.8

- Preserve each Claude model-specific weekly window alongside the all-models total.
- Explain missing project folders and unsupported Codex API-key quota queries before retrying.

- Save edited agent profiles asynchronously; show errors inside the dialog and close after success.

- Show normal Windows drive/UNC project paths and preserve drive-root separators.
- Create named workspaces by Enter or Create, waiting asynchronously for admitted metadata.
- Add confirmed workspace deletion with recoverable local app data and project-source protection.
- Name generic MCP server registration correctly in Settings and update the user manual.

## v0.1.7

- Generate plan no longer shows "Native response timed out" from the background status refresh while a large folder is being prepared.
- Windows: Claude capacity refresh runs the CLI in the project folder (previously cmd.exe fell back to System32) and reports "first-run setup is not finished" immediately instead of timing out; nothing is answered on the operator's behalf.

## v0.1.6

- Goals in other languages get an English slug proposed by the coordinator (for example `ladder_game`) instead of a romanized name.
- Enterprise capacity is shown as a percentage, like other windows (for example `64%/100%`); the tooltip keeps the raw amounts.
- Windows: a project folder picked with Browse, including Korean folder names, now saves like a typed path.
- Generate plan no longer shows a false timeout while preparing a large folder; the review area shows the coordinator's real output activity and warns when it stalls. The progress light blinks even with reduced motion.
- Settings → Agent tools guide… lists install commands for agentmemory, ponytail, rtk, graphify and cocoindex-code for Claude Code and Codex. Guidance only; the app installs and runs nothing.

## v0.1.5

- Plan and execute in ordinary project folders without initializing their Git repository. Private bounded change history supports reviewed diffs, tests and approved application; concurrent original edits are refused.
- Keep agent registration and edit forms responsive while executable, model and quota metadata is loading; preserve operator typing and profile changes.
- Run model and quota queries through one bounded background worker with a durable queue instead of locking the planning slot or unrelated controls.
- Avoid skipping another agent's quota refresh or consuming a cooldown when its refresh request was not accepted.
- Preserve account capacity when model catalogs refresh; catalog observations do not change account identity.
- Settle failed metadata without blocking settings; prune superseded private snapshots and compact completed working copies while retaining review evidence.
- Update the English user manual and README for workspace setup, asynchronous controls and ordinary-folder review/application.

## v0.1.4

- Open macOS folder and file pickers in the desktop app so project selection stays in front.
- Remove automatic Windows command windows from desktop startup and background Git, agent, metadata and messenger operations; retain CLI output and process containment.
- Display Codex Enterprise remaining allocation / allocated total; preserve unknown totals and individual scope.
- Reduce Settings button whitespace while preserving the visible gear.

## v0.1.3

- Slugs are named automatically from the goal (Korean is romanized, duplicates get `_2`…); the slug field is gone.
- **Answer tool requests from the messenger** (phase 2): listed approvers reply `allow <code>` / `deny <code>` in Slack, or a company relay returns HMAC-signed decisions; applied through the same exact, single-use approval as the Agent Room. Polling only while a request is waiting; tokens and secrets stay on this computer.

## v0.1.2

- Configure company model IDs directly for custom CLIs without a model-list command, with explicit operator provenance.

- **Messenger notifications** (phase 1): Slack Incoming Webhook or any company messenger webhook per workspace — plan ready, review/tool decisions, usage-limit waits and run outcomes. The webhook URL stays on this computer (never exported); messages are metadata-only unless the goal is allowed; delivery uses the system `curl` without new dependencies.
- Put MCP connection settings under the top-bar Settings menu; enlarge the gear and its pointer target.
- Fix Windows npm executable discovery and recover quota collection for existing extensionless profiles.
- Add a bounded, contained Windows ConPTY collector for Claude /usage; no model generation or automatic trust approval.
- Show original provider badges when desktop artwork is unavailable and repair legacy Windows icon alpha.

- Register OpenCode-shaped local and remote MCP servers in Workspace settings.
- Discover read-only tools and bind Jira/Confluence arguments to explicit goal references.
- Preview source text and provenance, approve selected evidence and retain immutable local snapshots for planning.
- Keep document bodies out of portable settings and shared memory; verify goal/settings/content digests before execution.
- Reuse one MCP session per server, bound responses/deadlines and close owned processes/sessions after each operation.
- Support authenticated local proxies and environment-backed remote headers; native OAuth browser authorization is not included.

## v0.1.1

### Added

- Goals become slugs with work folders; plans start with an analysis and run phase by phase (task → verification → completed); **Goals & progress** overview, **Plan next phase** and editable **Plan rules**.
- Slug folders live in `<repo>/.agentconductor/<slug>/` (or `~/.agentconductor/<workspace>/<slug>/` without a repository) and are kept out of `git status`.
- Repository published under the MIT license.
- Agent dialog: **Provider** comes first; switching provider replaces the Agent ID (unique suffix), executable path and options together and clears the old CLI's models. Renaming an agent keeps it as coordinator, in role rules and as fallback.
- **Settings** menu with interface language: English (default) or Korean; switches instantly and is remembered. UTF-8 is explicit everywhere (pages, release notes, Windows runners).
- Windows fixes found by the first real Windows CI run: release notes no longer fail under the cp1252 console encoding; executable lookup also searches `%APPDATA%\npm` and `%USERPROFILE%\.local\bin`.
- Agent dialog: choosing an alternate agent loads its models into **Alternate agent's model**, with an explanation of what the fallback does; the Settings button is a ⚙ icon.
- Exhausted quota shows when it returns, e.g. `0%/40% · 18:39`; the tooltip lists every window's reset time.

## v0.1.0

- Default quota refresh to five minutes, configurable per agent; update capacity silently in the background.
- Locate installed CLI executables automatically, provide native Browse, and standardize input control sizing.
- Use the project-owned AC mark for native application and executable identity.
- Clarify the three workspace steps and update actual native screenshots and the user manual.

### Added

- AgentConductor is published under the MIT license; `LICENSE` ships in every package.
- New workspaces start with an empty agent list; add the CLIs you use with **+ Add** (templates for Claude Code, Codex and OpenCode remain).
- Lower idle cost: windows refresh every 5 s when nothing is happening (1 s while work runs), pause while hidden, and no longer resend app icons every second (idle CPU ≈0.1 %).
- Workspace export now includes one `<slug>.zip` per goal with its progress; import verifies every archive before restoring.
- Quota collection in fresh workspaces, individual refresh icons and automatic two-minute
  refresh while idle; authenticated macOS Claude `/usage` metadata without generation.
- Compact Edit, Models reload and Remove icons in agent cards.

- Remaining quota beside paid agent names in the workspace and session lists, with
  window/source tooltips and explicit unavailable or stale states.

- Claude capacity in Agent Room: automatically collect provider-reported utilization and
  reset windows from owned task streams. **Check observed quota** preserves freshness
  without launching another prompt; absent utilization stays unavailable.

- When planning starts on a repository with uncommitted or untracked changes, a dialog now
  explains why a clean baseline is needed, lists the changed files and offers **Commit and
  continue** (one baseline commit on the current branch) or **Stop** (nothing changes).
- **Ignore and continue** in the same dialog for operating-system files (`.DS_Store`,
  `Thumbs.db`, `desktop.ini`, `._*`): excludes them locally without a commit, with a guide
  to ignore them in every repository.

- Planning progress: **Review the plan** shows the coordinator and elapsed time while a
  plan is generated, a *Plan ready* notice with scroll when done, and the exit status and
  log folder when planning fails.

- Task instructions are requested as readable *Goal / Inputs / Steps / Expected output /
  Done when* sections in the goal's language; the instructions box is twice as tall and
  grows with its content.

- **Agent roles** (step 2): decide per workspace which agent does verification,
  development, documentation, knowledge assetization and testing; the coordinator must
  follow these rules, and each planned task shows its role. Each role agent can be pinned
  to a model from its observed catalog; the engine applies that model to the task.
- The workspace screen is now numbered: **1. Workspace settings**, **2. Agent roles**,
  **3. Review the plan**.

- Agents added, edited or removed are saved immediately (other unsaved edits are kept
  as typed); removing an agent also updates the coordinator and role rules.

- Model lists load automatically once when a built-in agent is added; **Models** refreshes
  them. Lists update quietly in the role table without moving focus.
- Compact agent cards: provider, sessions, model count, permission capability and fallback
  at a glance.

- **Export settings… / Import settings…** in the Workspace menu: share name, coordinator,
  workspace mode, agents and role rules through `agentconductor.workspace.json` in the
  repository. No secrets, model lists, local paths, goals or plans are exported; imported
  sandbox claims must be re-confirmed on each machine.

### Fixed

- New workspaces showed no model lists; they now reuse lists already observed on this
  machine and fetch missing ones once in the background.
- Saving plan edits dropped each task's role.
- A generated plan was discarded when models were loaded or the workspace was saved with
  unrelated changes; it is now kept unless the goal, repository, workspace mode or an
  agent's execution settings change.

- Plans can have one to twelve tasks; **+ Add task** and **Remove task** in review.
- **Read plan…**: convert an existing Markdown plan document into tasks with the
  coordinator (read-only, role rules applied, completed items skipped).
- **Save workspace** moved into step 1.

### Changed

- **Open existing workspace…** now lists the workspaces saved in `~/.agentconductor`
  (including ones removed from the menu) instead of opening a folder dialog, so an
  arbitrary folder can no longer become a workspace by mistake.

## v0.1.0-preview.6

### Added

- **Write plan manually** — plan the three tasks yourself without any LLM coordinator; the
  draft uses the same validation and exact-digest approval as a generated plan.
- **Workspace name** above the goal, and a **Workspace** menu in the title bar: see every
  registered workspace by name, create one with **New workspace…** (name only), add an
  existing folder, switch in place, or remove an entry from the list (files are kept).
- **Workspace settings** section: goal first, then planning coordinator, then repository
  with a **Browse…** folder dialog next to the Git-worktree option.
- A workspace can be saved without a repository; only planning needs one.
- Release notes are now taken from this changelog.

### Fixed

- README and manual named the agent button "Register agent"; the button is **+ Add**.

### Known limitations

- Codex live re-verification waits for the account usage reset (2026-10-16).
- Claude account-capacity refresh is unavailable (no provider channel).

## v0.1.0-preview.5

### Added

- `memory-discover RUN PORT` finds and imports cooperative checkpoint reports from a local
  agentmemory server without memory IDs, scoped to each task's current address.

### Fixed

- Windows containment test no longer times out on a cold PowerShell start (test only).

## v0.1.0-preview.4

### Added

- A completed cooperative **handoff** now continues automatically on the agent's reviewed
  fallback once the relinquishing process has exited; without a fallback it stops as before.

### Verified

- Live Claude workers: an out-of-scope write is blocked before it happens; a rejected
  command never runs and that worker continues.
- Merge is refused when the target branch moved after approval.

## v0.1.0-preview.3

### Changed

- README rewritten as a short desktop-first introduction: what it does, how orchestration
  works, how roles are assigned, usage-limit handling and what you need.
- New user manual (`docs/user-manual.md`) with the desktop guide and every command; shipped
  in the package.

### Added

- Windows: provider icons are also found through Start Menu shortcuts.

## v0.1.0-preview.2

### Fixed

- Checksums and license notices are written with LF line endings on every platform, so
  `SHA256SUMS` verifies on macOS and Windows.

## v0.1.0-preview.1

### Added

- First pre-release: native desktop workspace and Agent Room for Claude Code, Codex,
  OpenCode and custom CLIs; three-task plans with exact approval; parallel sessions in a
  shared directory or Git worktrees; live tool approval; usage-limit waiting and reviewed
  fallback; reviewed lessons; macOS and Windows packages with licenses, SBOM and checksums.

- Compact agent quota groups show slash-separated percentages on one line beside their refresh icon; tooltips explain Enterprise Current/Total or provider duration labels.
