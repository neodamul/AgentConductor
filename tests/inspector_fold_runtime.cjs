// GitHub #146, amended by #394 / R-210: the docked task detail column starts closed, only a toggle opens it and a decision lights the toggle.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = (fs.readFileSync('src/ui/room.js', 'utf8') + fs.readFileSync('src/ui/room_render.js', 'utf8'));
const start = source.indexOf('// inspector-fold'), end = source.indexOf('// inspector-fold-end');
assert.ok(start >= 0 && end > start, 'room.js marks the inspector fold block');
assert.ok(!/localStorage/.test(source.slice(start, end)), 'the open/closed choice is not remembered');
const context = vm.createContext({ console });
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

// A toggle flips what is shown; a decision never opens the column, it only records its key.
const step = (s, ev) => run(`inspectorStep(${JSON.stringify(s)}, ${JSON.stringify(ev)})`);
let s = { open: false, key: '' };
s = step(s, { type: 'snapshot', key: 'a' });
assert.deepEqual(s, { open: false, key: 'a' }, 'a decision does not open the closed column');
s = step(s, { type: 'toggle' });
assert.deepEqual(s, { open: true, key: 'a' }, 'the toggle opens it');
s = step(s, { type: 'snapshot', key: 'b' });
assert.deepEqual(s, { open: true, key: 'b' }, 'a new decision keeps it as it is');
s = step(s, { type: 'toggle' });
assert.deepEqual(s, { open: false, key: 'b' });
s = step(s, { type: 'snapshot', key: '' });
assert.deepEqual(s, { open: false, key: '' }, 'the key clears when the decision is over');
console.log('inspector fold ok');
