// R-102 / DD-110 (verified for GitHub #349): moving an agent card saves the whole list in the new
// order, and a move is refused while the page is busy or the indexes are not cards.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/startup.js', 'utf8');
const line = source.split('\n').find(l => l.trim().startsWith('function moveAgent('));
assert.ok(line, 'startup.js defines moveAgent');
const saved = [];
let busy = false;
const context = vm.createContext({
  draft: { profiles: [{ id: 'claude' }, { id: 'codex' }, { id: 'opencode' }] },
  copy: v => JSON.parse(JSON.stringify(v)),
  busy: () => busy,
  saveProfiles: list => saved.push(list.map(p => p.id)),
});
vm.runInContext(line, context);
const move = (from, to) => vm.runInContext(`moveAgent(${from}, ${to})`, context);

move(2, 0);
assert.deepEqual(saved.pop(), ['opencode', 'claude', 'codex'], 'dragging the last card to the top');
move(0, 1);
assert.deepEqual(saved.pop(), ['codex', 'claude', 'opencode'], 'Move down swaps neighbours');
assert.deepEqual(context.draft.profiles.map(p => p.id), ['claude', 'codex', 'opencode'], 'the draft changes only through the saved list');
move(1, 1);
move(0, 7);
busy = true;
move(0, 1);
assert.equal(saved.length, 0, 'no save for a no-op, an index that is not a card, or a busy page');
