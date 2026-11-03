// R-118 / DD-127: when the window asks to close while a run works, the page shows one dialog with
// "Keep running" focused; each choice sends close_window with the operator's stop decision.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const lines = fs.readFileSync('src/ui/startup.js', 'utf8').split('\n');
const at = lines.findIndex(l => l.includes('window.__acCloseRequested='));
assert.ok(at >= 0, 'startup.js answers the native close request');
const block = lines.slice(at, at + 3).join('\n');
const html = fs.readFileSync('src/ui/startup.html', 'utf8');
for (const id of ['start-close-dialog', 'start-close-keep', 'start-close-stop']) assert.ok(html.includes(`id="${id}"`), id);

const element = () => ({ open: false, focused: false, showModal() { this.open = true; }, close() { this.open = false; }, focus() { this.focused = true; } });
const nodes = { 'close-dialog': element(), 'close-keep': element(), 'close-stop': element() };
const sent = [], notices = [];
let fail = false;
const context = vm.createContext({
  window: {},
  q: id => nodes[id],
  request: packet => { sent.push(packet); return fail ? Promise.reject(new Error('refused')) : Promise.resolve({ closing: true }); },
  notice: (text, error) => notices.push([text, error]),
});
vm.runInContext(block, context);

context.window.__acCloseRequested();
assert.ok(nodes['close-dialog'].open && nodes['close-keep'].focused, 'the dialog opens with Keep running focused');
context.window.__acCloseRequested();
nodes['close-keep'].onclick();
nodes['close-stop'].onclick();
assert.deepEqual(JSON.parse(JSON.stringify(sent)), [{ kind: 'close_window', stop: false }, { kind: 'close_window', stop: true }]);

(async () => {
  fail = true;
  await nodes['close-stop'].onclick();
  assert.equal(nodes['close-dialog'].open, false, 'a refused close leaves the window usable');
  assert.deepEqual(notices.at(-1), ['refused', true]);
})();
