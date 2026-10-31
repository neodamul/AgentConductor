// Fake native bridge. SIM.home / SIM.room are the snapshots; SIM.reply[kind] answers other
// requests; SIM.hang[kind] keeps a request pending until SIM.release(kind) (or forever).
(() => {
  const now = Date.now();
  const profiles = [{ id: 'claude', parser: 'claude', executable: '/usr/local/bin/claude', args: [], max_sessions: 3, permission_mode: 'native_callback', catalog: { models: ['opus'], source: 'sim', queried_at: 1, cli_version: '1' } },
    { id: 'codex', parser: 'codex', executable: '/usr/local/bin/codex', args: [], max_sessions: 3, permission_mode: 'native_sandbox', catalog: { models: ['gpt'], source: 'sim', queried_at: 1, cli_version: '1' } }];
  const config = { name: 'sim', repository: '/tmp/sim-repo', goal: 'Build the thing', coordinator: 'claude', workspace_mode: 'worktree', roles: {}, profiles, mcp: {}, goal_sources: {} };
  const plan = { goal: config.goal, workspace_mode: 'worktree', agents: profiles, tasks: [
    { id: 'task-1', agent: 'codex', model: null, prompt: 'Write the parser', paths: ['src/a'], depends_on: [], test_command: ['true'] },
    { id: 'task-2', agent: 'claude', model: null, prompt: 'Write the docs', paths: ['docs'], depends_on: ['task-1'], test_command: ['true'] },
    { id: 'task-3', agent: 'codex', model: null, prompt: 'Write the tests', paths: ['tests'], depends_on: [], test_command: ['true'] }] };
  const task = (id, status, extra = {}) => ({ id, status, attempt: status === 'pending' ? 0 : 1, started_ms: status === 'pending' ? null : now - 90000, finished_ms: null, session_id: null, error: null, timeouts: 0, retry_not_before: 0, ...extra });
  window.SIM = {
    log: [], hang: {}, waiters: {}, reply: {},
    home: { revision: 'r1', chat_revision: 'c1', config, proposal: null, job: null, quota_job: null, metadata_job: null, metadata_queue: [], root: '/tmp/sim-desktop', templates: [], run_available: false, run_state: null, quotas: {}, quota_badges: {}, quota_refresh_due: {}, slugs: [], language: 'ko', imports: null, source_notes: null, workspace_blank: false, chat: null, chat_job: null, optimize: null },
    room: { revision: 'x1', report: { state: 'running', repo: '/tmp/sim-repo', branch: 'main', plan, tasks: [task('task-1', 'running'), task('task-2', 'pending'), task('task-3', 'running')], tests: [], candidate: null, error: null, direct_workspace: false, folder: null }, activity: {}, quotas: {}, job: null, tool_requests: [], coordinator: 'claude', decision_count: 0, tool_decision_wait_secs: 600, run_dir: '/tmp/sim-run', phase: null, owned_changes: [], availability_timer: [], availability_timer_error: null, runner: 'present', stopping: false, verification: null },
    plan, task, now,
    proposal: phase => ({ phase, plan: phase === 'ready' ? plan : null, digest: 'd1', repository_base: 'abcdef1234567890', directory: '/tmp/p', run: '/tmp/sim-run', task_profiles: [] }),
    release(kind) { for (const f of this.waiters[kind] || []) f(); this.waiters[kind] = []; },
  };
  const defaults = { workspaces: () => ({ name: 'sim', current: '/tmp/sim-desktop', registered: [] }), chat_history: () => ({ turns: [] }), check_update: () => ({ phase: 'idle', current: 'sim', newer: false }), update_status: () => ({ phase: 'idle', current: 'sim', newer: false }), open_run: () => ({ opened: true, run: 'run-1' }), coordinator: () => ({ history: [], judgments: [], stdout: '', stderr: '' }), task: p => ({ task: SIM.room.report.tasks[p.index], events: [], decisions: [], knowledge: [], step_lessons: [], usage: null, stdout: '', stderr: '' }) };
  window.__acRequest = body => {
    const p = JSON.parse(body), kind = p.kind === 'snapshot' ? (p.room ? 'room_snapshot' : 'snapshot') : p.kind;
    SIM.log.push({ at: Date.now(), kind, p });
    const answer = () => { const f = SIM.reply[kind]; if (f instanceof Error) throw f; const v = f ? f(p) : kind === 'snapshot' ? SIM.home : kind === 'room_snapshot' ? SIM.room : defaults[kind] ? defaults[kind](p) : { accepted: true, kind }; return JSON.stringify(v); };
    if (SIM.hang[kind]) return new Promise((resolve, reject) => (SIM.waiters[kind] ||= []).push(() => { try { resolve(answer()); } catch (e) { reject(e); } }));
    return new Promise((resolve, reject) => setTimeout(() => { try { resolve(answer()); } catch (e) { reject(e); } }, 20));
  };
})();
// Probes: what the operator sees in the given elements ("(hidden)" when not on screen) and which buttons are marked busy.
window.PROBE = ids => { ids = ids || ['start-phase', 'start-status', 'start-step-progress', 'start-plan-progress', 'start-run-progress', 'start-notice', 'start-optimize-status']; const v = id => { const e = document.getElementById(id); if (!e) return null; for (let n = e; n; n = n.parentElement) if (n.hidden || getComputedStyle(n).display === 'none') return '(hidden)'; return e.textContent.trim().replace(/\s+/g, ' ').slice(0, 220); }; const out = {}; for (const id of ids) out[id] = v(id); out.busy = [...document.querySelectorAll('.is-busy')].map(b => b.id || b.textContent.trim()).join(','); return out; };
window.ROOM = () => PROBE(['run-banner-state', 'run-banner-progress', 'run-banner-next', 'run-banner-attention', 'job-state', 'task-status', 'task-activity', 'connection', 'notice']);
window.wait = ms => new Promise(r => setTimeout(r, ms));
// A hidden browser pane fires no dialog close event; CONFIRM closes a dialog as the confirm button would.
window.CONFIRM = id => { const d = document.getElementById(id); d.close('confirm'); d.dispatchEvent(new Event('close')); };
// PRESET(code) reloads the page with `code` run before the app starts (state at opening, e.g. a job already running).
try { const p = sessionStorage.getItem('SIM_PRESET'); if (p) (0, eval)(p); } catch (e) { console.log('preset', e); }
window.PRESET = code => { sessionStorage.setItem('SIM_PRESET', code); location.reload(); };
