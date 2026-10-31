// GitHub #273 (DD-208): every waiting state of the pages has a visible, current sign.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const room = fs.readFileSync('src/ui/room.js', 'utf8'), startup = fs.readFileSync('src/ui/startup.js', 'utf8'), i18n = fs.readFileSync('src/ui/i18n.js', 'utf8');
const grab = (source, head) => { const s = source.indexOf(head); assert.ok(s >= 0, `${head} exists`); return source.slice(s, source.indexOf('\n', s)) + '\n'; };
const catalog = JSON.parse(fs.readFileSync('src/ui/messages/en.json', 'utf8'));
const acMessage = (id, v = {}) => { assert.ok(catalog[id] !== undefined, `${id} is in the catalog`); return catalog[id].replace(/\{(\w+)\}/g, (_, k) => String(v[k])); };
const context = vm.createContext({ acMessage });
vm.runInContext(['function formatAge(', 'function pendingReason(', 'function verificationText(', 'function runAttention('].map(h => grab(room, h)).join('') + grab(i18n, 'const AC_STALE=') + grab(startup, 'function startText(') + 'this.AC_STALE=AC_STALE;', context);
const call = (fn, ...args) => vm.runInContext(`${fn}(...${JSON.stringify(args)})`, context);

// A pending task says what it waits for.
const plan = { agents: [{ id: 'a', executable: '/bin/a', max_sessions: 1 }, { id: 'b', executable: '/bin/b', quota_group: 'team', max_sessions: 2 }],
  tasks: [{ id: 'one', agent: 'a', depends_on: [] }, { id: 'two', agent: 'a', depends_on: ['one'] }, { id: 'three', agent: 'b', depends_on: [] }, { id: 'four', agent: 'a', depends_on: [] }] };
const report = (tasks, extra = {}) => ({ plan, tasks, account_restrictions: {}, ...extra });
const now = 1_000_000_000_000;
let r = report([{ id: 'one', status: 'running' }, { id: 'two', status: 'pending' }, { id: 'three', status: 'pending' }, { id: 'four', status: 'pending' }]);
assert.match(call('pendingReason', r, 1, now), /one/, 'an unfinished dependency is named');
assert.equal(call('pendingReason', r, 0, now), '', 'a running task has no pending reason');
assert.match(call('pendingReason', r, 3, now), /1/, 'the session limit of the agent is named');
assert.equal(call('pendingReason', r, 2, now), '', 'a task about to start has no reason');
r = report([{ id: 'one', status: 'completed' }, { id: 'two', status: 'pending', timeouts: 1, retry_not_before: now / 1000 + 90 }, { id: 'three', status: 'pending' }, { id: 'four', status: 'completed' }], { account_restrictions: { team: { reason: 'quota' } } });
assert.match(call('pendingReason', r, 1, now), /1m 30s/, 'the automatic retry time is counted down');
assert.equal(call('pendingReason', r, 2, now), acMessage('room.pending_account'), 'an account restriction on the quota group is named');

// The integration verification names the test in progress, its time and its output age.
assert.equal(call('verificationText', null, now), '');
assert.equal(call('verificationText', { stage: 'candidate' }, now), acMessage('room.verification_preparing'));
const step = call('verificationText', { stage: 'test', index: 1, total: 3, task: 'two', started_ms: now - 65000, output: { bytes: 10, updated_ms: now - 4000 } }, now);
for (const part of ['2', '3', 'two', '1m 05s', '4s']) assert.ok(step.includes(part), `${part} in ${step}`);
assert.ok(call('verificationText', { stage: 'test', index: 0, total: 1, task: 'one', started_ms: now, output: { bytes: 0, updated_ms: null } }, now).includes(acMessage('room.activity_no_output')));

// A stop in progress and a run without its process are said; a busy action or another state is not.
const snap = (state, extra) => ({ report: { state }, job: null, ...extra });
assert.equal(call('runAttention', snap('running', { stopping: true, runner: 'present' })), acMessage('room.stopping_run'));
assert.equal(call('runAttention', snap('running', { stopping: false, runner: 'absent' })), acMessage('room.runner_absent'));
assert.equal(call('runAttention', snap('running', { stopping: false, runner: 'absent', job: { state: 'running', kind: 'resume' } })), '', 'a resume that is starting the run is not a dead run');
assert.equal(call('runAttention', snap('running', { stopping: false, runner: 'unknown' })), '', 'an unknown presence is not reported as absent');
assert.equal(call('runAttention', snap('stopped', { stopping: true, runner: null })), '', 'a finished stop is not in progress');

// The pages say how long no new state came, after 5 s.
assert.equal(context.AC_STALE(now - 4000, now), '');
assert.equal(context.AC_STALE(0, now), '', 'before the first snapshot nothing is claimed');
assert.ok(context.AC_STALE(now - 41000, now).includes('41'));

// The start line counts from the start job's recorded start, else from the click.
const line = call('startText', { kind: 'start', state: 'running', started_ms: now - 125000 }, now, now - 1000);
assert.ok(line.includes('2:05'), line);
assert.ok(call('startText', null, now, now - 7000).includes('0:07'));

// The start is sent to the right element (it threw before sending since #38).
assert.ok(!startup.includes("'start-run-progress')"), 'q() adds the start- prefix itself');
console.log('progress runtime: pending reasons, verification, run attention, staleness and start time are shown');
