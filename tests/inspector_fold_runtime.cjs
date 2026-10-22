// GitHub #146: the task detail column folds, remembers the choice and opens for a decision.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
const start = source.indexOf('// inspector-fold'), end = source.indexOf('// inspector-fold-end');
assert.ok(start >= 0 && end > start, 'room.js marks the inspector fold block');
const store = new Map();
let broken = false;
const localStorage = {
  getItem: k => { if (broken) throw new Error('blocked'); return store.has(k) ? store.get(k) : null; },
  setItem: (k, v) => { if (broken) throw new Error('blocked'); store.set(k, String(v)); },
};
const context = vm.createContext({ localStorage, console });
vm.runInContext(source.slice(start, end), context);
const run = code => { const v = vm.runInContext(code, context); return v === undefined ? undefined : JSON.parse(JSON.stringify(v)); };
const snap = (state, extra = {}) => JSON.stringify({ report: { state, error: '', candidate: { commit: 'c1' }, tasks: [], ...extra.report }, tool_requests: extra.tools || [] });

// What counts as a decision, and that each decision has its own key.
assert.equal(run(`decisionKey(${snap('running')})`), '');
assert.equal(run(`decisionKey(${snap('completed')})`), '');
assert.notEqual(run(`decisionKey(${snap('waiting_approval')})`), '');
assert.notEqual(run(`decisionKey(${snap('blocked')})`), '');
assert.notEqual(run(`decisionKey(${snap('waiting_tool_approval')})`), '');
assert.notEqual(run(`decisionKey(${snap('failed', { report: { error: 'out-of-scope change: a' } })})`), '', 'a scope failure needs a person');
assert.equal(run(`decisionKey(${snap('failed', { report: { error: 'integration tests failed' } })})`), '', 'a plain failure is not a decision');
assert.notEqual(run(`decisionKey(${snap('running', { tools: [{ id: 't1', status: 'pending' }] })})`), '', 'a pending tool request is a decision');
assert.equal(run(`decisionKey(${snap('running', { tools: [{ id: 't1', status: 'approved' }] })})`), '');
assert.notEqual(run(`decisionKey(${snap('waiting_approval')})`), run(`decisionKey(${snap('waiting_approval', { report: { candidate: { commit: 'c2' } } })})`), 'a new candidate is a new decision');
assert.equal(run('decisionKey(null)'), '');

// The folded column opens once per decision and then follows the operator's choice.
const step = (s, ev) => run(`inspectorStep(${JSON.stringify(s)}, ${JSON.stringify(ev)})`);
let s = { pref: true, open: false, key: '' };
s = step(s, { type: 'snapshot', key: '' });
assert.deepEqual(s, { pref: true, open: false, key: '' }, 'no decision: stays folded');
s = step(s, { type: 'snapshot', key: 'a' });
assert.deepEqual(s, { pref: true, open: true, key: 'a' }, 'a decision opens the folded column');
s = step(s, { type: 'toggle' });
assert.deepEqual(s, { pref: true, open: false, key: 'a' }, 'folding during the decision is respected');
s = step(s, { type: 'snapshot', key: 'a' });
assert.equal(s.open, false, 'the same decision does not reopen it');
s = step(s, { type: 'snapshot', key: 'b' });
assert.equal(s.open, true, 'a different decision opens it again');
s = step(s, { type: 'snapshot', key: '' });
assert.deepEqual(s, { pref: true, open: false, key: '' }, 'after the decision the preference returns');

// Unfolding is remembered too, and an unfolded column returns unfolded.
s = step({ pref: true, open: false, key: '' }, { type: 'toggle' });
assert.deepEqual(s, { pref: false, open: true, key: '' });
s = step(step(s, { type: 'snapshot', key: 'a' }), { type: 'snapshot', key: '' });
assert.deepEqual(s, { pref: false, open: true, key: '' });

// The preference survives a new page and a blocked storage leaves the column open.
assert.equal(run('loadInspectorPref()'), false, 'open by default');
run('saveInspectorPref(true)');
assert.equal(run('loadInspectorPref()'), true);
run('saveInspectorPref(false)');
assert.equal(run('loadInspectorPref()'), false);
broken = true;
assert.equal(run('loadInspectorPref()'), false, 'blocked storage: open, no error');
run('saveInspectorPref(true)');
console.log('inspector fold ok');
