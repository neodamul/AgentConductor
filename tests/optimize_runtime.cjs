// GitHub #180: Optimize prompt requests a rewrite, shows the proposal once the job completes, and lets the operator use or drop it.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/optimize.js', 'utf8');
const node = () => ({ children: [], hidden: false, disabled: false, value: '', textContent: '', open: false, attrs: {},
  replaceChildren(...n) { this.children = n; }, showModal() { this.open = true; }, close() { this.open = false; } });
const els = {}; // Like startup.js, q(id) looks up the element with id start-<id>.
for (const id of ['goal', 'optimize-button', 'optimize-status', 'optimize', 'optimize-goal', 'optimize-changes', 'optimize-changes-title',
  'optimize-questions', 'optimize-questions-title', 'optimize-use', 'optimize-keep']) els['start-' + id] = node();
const q = id => { if (!els['start-' + id]) throw new Error('no element start-' + id); return els['start-' + id]; };
const calls = [], notices = [], applied = []; let current = { revision: 'rev', job: null, optimize: null }, busy = false, reply = { id: 'job-1' };
const context = vm.createContext({ document: { createElement: node }, console,
  acMessage: (id, v = {}) => id + (v.value1 ? ':' + v.value1 : ''), AC_I18N: { text: (n, f) => { n.textContent = f(); } } });
vm.runInContext(source + ';this.optimizeUi=optimizeUi;', context);
(async () => {
  const ui = context.optimizeUi({ q, request: async p => { calls.push(p); if (reply instanceof Error) throw reply; return reply; },
    state: () => current, notice: (m, e) => notices.push([m, e]), busy: () => busy, applyGoal: t => applied.push(t) });
  ui.update();
  assert.equal(q('optimize-button').disabled, true, 'an empty goal cannot be optimized');
  q('goal').value = 'make it faster'; ui.update();
  assert.equal(q('optimize-button').disabled, false);
  busy = true; ui.update();
  assert.equal(q('optimize-button').disabled, true, 'another operation blocks it');
  busy = false; ui.update();
  await q('optimize-button').onclick();
  assert.deepEqual(JSON.parse(JSON.stringify(calls)), [{ kind: 'optimize_goal', revision: 'rev', goal: 'make it faster' }]);
  assert.equal(q('optimize-button').disabled, true, 'blocked while the coordinator works');
  assert.match(q('optimize-status').textContent, /^startup\.optimize_working 0:0\d$/, 'the status counts the time (GitHub #273)');
  current = { revision: 'rev', job: { id: 'job-1', state: 'completed' },
    optimize: { job: 'job-1', proposal: { optimized: 'Speed up checkout; verify with npm run bench', changes: ['added a check'], questions: ['Which browser?'] } } };
  ui.update();
  assert.equal(q('optimize').open, true, 'the proposal opens in the dialog');
  assert.equal(q('optimize-goal').textContent, 'Speed up checkout; verify with npm run bench');
  assert.equal(q('optimize-changes').children[0].textContent, 'added a check');
  assert.equal(q('optimize-questions').children[0].textContent, 'Which browser?');
  assert.equal(q('optimize-button').disabled, false);
  assert.equal(q('optimize-status').textContent, '');
  q('optimize-keep').onclick();
  assert.equal(q('optimize').open, false);
  assert.deepEqual(applied, [], 'keeping leaves the goal alone');
  ui.update();
  assert.equal(q('optimize').open, false, 'a proposal is shown once');
  // Use this goal applies the proposal text and closes the dialog.
  reply = { id: 'job-2' }; await q('optimize-button').onclick();
  current = { revision: 'rev', job: { id: 'job-2', state: 'completed' },
    optimize: { job: 'job-2', proposal: { optimized: 'Second', changes: [], questions: [] } } };
  ui.update();
  assert.equal(q('optimize-changes').hidden, true, 'empty lists are hidden');
  assert.equal(q('optimize-questions-title').hidden, true);
  q('optimize-use').onclick();
  assert.deepEqual(applied, ['Second']);
  assert.equal(q('optimize').open, false);
  // A failed job reports the reason and does not open the dialog.
  reply = { id: 'job-3' }; await q('optimize-button').onclick();
  current = { revision: 'rev', job: { id: 'job-3', state: 'failed', error: 'no coordinator' }, optimize: null };
  ui.update();
  assert.equal(q('optimize').open, false);
  assert.deepEqual(notices.at(-1), ['startup.optimize_error:no coordinator', true]);
  assert.equal(q('optimize-button').disabled, false);
  // A refused request reports the error and re-enables the button.
  reply = new Error('another operation is running'); await q('optimize-button').onclick();
  assert.deepEqual(notices.at(-1), ['another operation is running', true]);
  assert.equal(q('optimize-button').disabled, false);
  // GitHub #273: a rewrite still running when the page reopens is followed and its time counted from its start.
  const reopened = context.optimizeUi({ q, request: async () => reply, state: () => current, notice: (m, e) => notices.push([m, e]), busy: () => false, applyGoal: t => applied.push(t) });
  current = { revision: 'rev', job: { id: 'job-4', kind: 'optimize', state: 'running', started_ms: Date.now() - 125000 }, optimize: null };
  reopened.update();
  assert.equal(q('optimize-button').disabled, true);
  assert.match(q('optimize-status').textContent, /2:0[5-6]$/);
  current = { revision: 'rev', job: { id: 'job-4', kind: 'optimize', state: 'completed' }, optimize: { job: 'job-4', proposal: { optimized: 'Fourth', changes: [], questions: [] } } };
  reopened.update();
  assert.equal(q('optimize-goal').textContent, 'Fourth', 'its proposal is shown');
  console.log('optimize runtime ok');
})().catch(e => { console.error(e); process.exitCode = 1; });
