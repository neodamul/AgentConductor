// R-120 / DD-129 (GitHub #385): consecutive thinking updates read as one Activity line, and the run
// banner states how often the run was interrupted and recovered.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const lines = fs.readFileSync('src/ui/room_activity.js', 'utf8').split('\n');
const pick = start => { const l = lines.find(x => x.startsWith(start)); assert.ok(l, start); return l; };
const messages = require('../src/ui/messages/en.json');
const context = vm.createContext({
  acMessage: (id, v = {}) => { assert.ok(messages[id], id); return messages[id].replace(/\{(\w+)\}/g, (_, k) => v[k]); },
  pretty: s => String(s).replaceAll('_', ' '),
});
vm.runInContext([pick('const thinkingUpdate='), pick('function collapseThinking('), pick('function interruptionText(')].join('\n') + '\nthis.out=[collapseThinking, interruptionText];', context);
const [collapse, interruption] = context.out;
const plain = x => JSON.parse(JSON.stringify(x));

const think = seq => ({ seq, payload: { type: 'thinking_tokens' } });
const said = seq => ({ seq, payload: { type: 'assistant' } });
assert.deepEqual(plain(collapse([said(1), think(2), think(3), think(4), said(5), think(6)])), [
  { event: said(1) },
  { thinking: { count: 3, first: 2, last: 4 } },
  { event: said(5) },
  { thinking: { count: 1, first: 6, last: 6 } },
]);
assert.deepEqual(plain(collapse([{ seq: 1, payload: { type: 'system', subtype: 'thinking_tokens' } }])), [{ thinking: { count: 1, first: 1, last: 1 } }]);
assert.deepEqual(plain(collapse([])), []);

assert.equal(interruption([]), '');
assert.equal(interruption(undefined), '');
const text = interruption([{ at: 1000, status: 'resumed' }, { at: Date.UTC(2026, 10, 3, 12, 34), status: 'retry_limit' }]);
assert.ok(text.includes('2') && text.includes('retry limit'), text);
