// R-114 / DD-122 (verified for GitHub #349): beside the workspace name, the run screen names the
// phase it executes (n of m and its name) with what it covers, and hides the box without a phase.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
const lines = source.split('\n');
const at = lines.findIndex(l => l.includes('DD-122: the titlebar names the run'));
assert.ok(at >= 0, 'room.js marks the run phase block');
const block = lines.slice(at, at + 2).join('\n');

const boxes = {};
const box = () => ({ hidden: null, children: [], title: '', append(...n) { this.children.push(...n); } });
const context = vm.createContext({
  document: { getElementById: id => boxes[id] || null },
  AC_I18N: { replace: b => { b.children = []; }, bind: (b, f, attr) => { b[attr || 'text'] = f(); } },
  acMessage: (id, v) => `${id}:${v.value1}/${v.value2}:${v.value3}`,
  node: (tag, text, cls) => ({ tag, text: typeof text === 'function' ? text() : text, cls }),
});
const render = phase => {
  boxes['run-phase'] = box();
  boxes['start-run-phase'] = box();
  context.snapshot = { phase };
  vm.runInContext(`{\n${block}\n}`, context);
  return boxes['run-phase'];
};

const shown = render({ slug: 'ladder', phase: 'build', description: 'Build the board',
  phases: [{ id: 'analysis', name: 'Analysis' }, { id: 'build', name: 'Build' }, { id: 'ship' }] });
assert.equal(shown.hidden, false);
assert.equal(shown.children[0].text, 'room.phase:2/3:Build', 'phase n of m and its name');
assert.equal(shown.children[1].text, 'Build the board', 'what the phase covers');
assert.equal(shown.title, 'ladder · Build the board', 'the full text is in the tooltip');
assert.equal(boxes['start-run-phase'].hidden, false, 'the workspace screen shows it too');

assert.equal(render(null).hidden, true, 'no phased run: hidden');
assert.equal(render({ phase: 'gone', phases: [{ id: 'build' }] }).hidden, true, 'unknown phase: hidden');
