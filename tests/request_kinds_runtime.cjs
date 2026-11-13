// GitHub #301 (R-202 / QAS-200): every request kind the pages send is classified once, so a wait that
// shows nothing cannot be added unnoticed. A new kind fails this test until the author decides which
// class it is in, which is the moment to decide how its wait is shown.
//   job     starts a background job; the page must have a progress rendering for it (named below)
//   refresh periodic or read-only; the only kinds AC_WAIT leaves quiet
//   direct  everything else; AC_WAIT marks the pressed button and names the request after 1 s
const fs = require('node:fs'), assert = require('node:assert/strict');
const read = f => fs.readFileSync(`src/ui/${f}`, 'utf8').replace(/\r\n/g, '\n');
const pages = ['startup.js', 'room.js', 'mcp.js', 'chat.js', 'optimize.js', 'manual.js', 'team_briefing.js'];
const found = new Set();
for (const page of pages) for (const m of read(page).matchAll(/\bkind:\s*'([a-z_]+)'/g)) found.add(m[1]);

const ENVELOPE = ['action']; // the room wraps its actions as { kind: 'action', action: { kind } }
const REFRESH = ['snapshot', 'client_timeout', 'task', 'coordinator', 'chat_history', 'workspaces', 'mcp_source_suggestions'];
const JOBS = [
  { kinds: ['prepare'], file: 'startup.js', text: "kind==='plan'" },
  { kinds: ['start'], file: 'startup.js', text: "kind==='start'" },
  { kinds: ['optimize_goal'], file: 'optimize.js', text: "kind==='optimize'" },
  { kinds: ['mcp_preview', 'mcp_discover'], file: 'mcp.js', text: "kind==='mcp'" },
  { kinds: ['chat'], file: 'chat.js', text: 'chat_job' },
  { kinds: ['discover'], file: 'startup.js', text: 'kind===`catalog:${p.id}`' },
  { kinds: ['apply_update'], file: 'startup.js', text: "updateSince('installing')" },
  // Room actions run as controller jobs shown in the banner; the usage refresh marks its button (and counts its time once #297 lands).
  { kinds: ['approve', 'merge', 'reject', 'revise', 'recover', 'resume', 'reassign', 'schedule', 'model', 'stop', 'coordinator_request', 'scope_recovery'], file: 'room.js', text: 'room.in_progress' },
  { kinds: ['quota'], file: 'startup.js', text: "metadataBusy('quota',p.id)" },
];
// A job that has its own progress line is also kept out of the generic waiting line.
const QUIET_JOBS = ['quota'];
const UPDATE_POLLS = ['update_status', 'check_update']; // sent as strings through updateRequest()
const DIRECT = ['report_bug','release_notes','workspace_briefing','baseline_status','briefing','cache_clear','cache_preview','catalog','chat_reset','close_window','commit_baseline','create_workspace','delete_workspace','draft','edit','evidence','export_settings','forget_workspace','home','ignore_os_files','import_settings','manual_document','mcp_apply','mcp_save','messenger','next_phase','open_run','open_workspace','pick_executable','pick_file','pick_folder','preview_settings','read_plan','resolve_executable','save','save_messenger','save_plan_rules','save_profiles','saved_workspaces','scope_failure','scope_preview','set_language','set_signal_transport','set_theme','set_update_settings','signal_transport','stop_planning','switch_workspace','test_messenger','undo_amendment','update_settings'];

const classes = { ENVELOPE, REFRESH, JOBS: JOBS.flatMap(j => j.kinds), DIRECT };
const owners = new Map();
for (const [name, kinds] of Object.entries(classes)) for (const kind of kinds) {
  assert.ok(!owners.has(kind), `${kind} is classified twice (${owners.get(kind)} and ${name})`);
  owners.set(kind, name);
}
const unclassified = [...found].filter(kind => !owners.has(kind));
assert.deepEqual(unclassified, [], `classify the new request kind(s) in tests/request_kinds_runtime.cjs: ${unclassified}`);
const stale = [...owners.keys()].filter(kind => !found.has(kind));
assert.deepEqual(stale, [], `these kinds are no longer sent, remove them: ${stale}`);

// The kinds AC_WAIT leaves quiet are exactly the refreshes, the usage job and the update polls.
const quiet = /const quiet=new Set\(\[([^\]]*)\]\)/.exec(read('i18n.js'));
assert.ok(quiet, 'AC_WAIT keeps a quiet list');
assert.deepEqual([...quiet[1].matchAll(/'([a-z_]+)'/g)].map(m => m[1]).sort(), [...REFRESH, ...QUIET_JOBS, ...UPDATE_POLLS].sort(),
  'a kind is quiet only when it is a refresh, the usage job or an update poll');

// Every job has a progress rendering in the page code.
for (const { kinds, file, text } of JOBS) assert.ok(read(file).includes(text), `${kinds} has no progress rendering: ${file} lacks ${text}`);
console.log(`request kinds: ${found.size} classified (${REFRESH.length} refresh, ${JOBS.flatMap(j => j.kinds).length} job, ${DIRECT.length} direct)`);
