// GitHub #122: the recovery panel previews, requires approval of that exact preview, then applies it.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
const start = source.indexOf('function scopeLines('), endMarker = "$('scope-task').onchange = scopeFillTask; $('scope-paths').oninput = $('scope-test').oninput = scopeReset;";
const end = source.indexOf(endMarker) + endMarker.length;
assert.ok(start >= 0 && end > start, 'room.js defines the recovery panel');
const node = () => ({ children: [], hidden: false, disabled: false, value: '', textContent: '', opened: false,
  replaceChildren(...n) { this.children = n; }, showModal() { this.opened = true; }, close() { this.opened = false; } });
const els = {}; const $ = id => { if (!els[id]) throw new Error('no element ' + id); return els[id]; };
for (const id of ['scope-review', 'scope-dialog', 'scope-task', 'scope-paths', 'scope-test', 'scope-preview', 'scope-approve', 'scope-offending', 'scope-tasks', 'scope-stray', 'scope-stray-box', 'scope-proposal']) els[id] = node();
const calls = [], acts = [], notices = [];
const failure = { offending: ['extra.txt'], tasks: [{ id: 'one.txt', paths: ['one.txt'], test_command: ['mock', 'test'] }], stray: [{ path: 'docs/new.md', exists_at_original: false }] };
const context = vm.createContext({ $, document: { createElement: node }, acMessage: (id, v = {}) => id + (v.value1 ? ':' + v.value1 : '') + (v.value2 ? '>' + v.value2 : ''),
  notice: (m, e) => notices.push([typeof m === 'function' ? m() : m, e]), console,
  request: async p => { calls.push(p); if (p.kind === 'scope_failure') return failure; return { task: 'one.txt', previous_paths: ['one.txt'], paths: p.paths, previous_test_command: ['mock', 'test'], test_command: p.test_command || ['mock', 'test'], restore: [{ path: 'docs/new.md', action: 'restore' }], digest: 'd1' }; },
  act: async a => { acts.push(a); return true; }, snapshot: null });
vm.runInContext(source.slice(start, end), context);
const j = x => JSON.parse(JSON.stringify(x));
(async () => {
  assert.equal(vm.runInContext("scopeOffered({report:{state:'failed',error:'out-of-scope change: a'}})", context), true);
  assert.equal(vm.runInContext("scopeOffered({report:{state:'failed',error:'integration test failed'}})", context), false);
  assert.equal(vm.runInContext("scopeOffered({report:{state:'waiting_approval',error:'out-of-scope change: a'}})", context), false);
  await els['scope-review'].onclick();
  assert.equal(els['scope-dialog'].opened, true);
  assert.equal(els['scope-paths'].value, 'one.txt\nextra.txt', 'approved paths plus the offending files are proposed');
  assert.equal(els['scope-approve'].disabled, true, 'nothing is approvable before a preview');
  els['scope-task'].value = 'one.txt';
  await els['scope-preview'].onclick();
  assert.deepEqual(j(calls.at(-1)), { kind: 'scope_preview', task: 'one.txt', paths: ['one.txt', 'extra.txt'] }, 'an unchanged test command is not sent');
  assert.equal(els['scope-approve'].disabled, false);
  assert.ok(els['scope-proposal'].textContent.includes('docs/new.md'));
  els['scope-paths'].value = 'extra.txt'; els['scope-paths'].oninput();
  assert.equal(els['scope-approve'].disabled, true, 'editing invalidates the reviewed preview');
  els['scope-test'].value = 'true'; await els['scope-preview'].onclick();
  assert.deepEqual(j(calls.at(-1)).test_command, ['true']);
  await els['scope-approve'].onclick();
  assert.deepEqual(j(acts[0]), { kind: 'scope_recovery', task: 'one.txt', paths: ['extra.txt'], test_command: ['true'], digest: 'd1' });
  assert.equal(els['scope-dialog'].opened, false);
  console.log('scope recovery runtime ok');
})().catch(e => { console.error(e); process.exitCode = 1; });
