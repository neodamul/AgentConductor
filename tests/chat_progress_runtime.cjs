// GitHub #514: a running coordinator chat shows its CLI's recent tool lines and the age of its last output inside the one working bubble;
// the 1 s timer only updates the status label; a reader who scrolled up is not pulled down; the answer replaces the bubble.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/chat.js', 'utf8');
const node = () => ({ children: [], hidden: false, disabled: false, value: '', textContent: '', className: '', scrollTop: 0, scrollHeight: 1000, clientHeight: 200, attrs: {}, handlers: {},
  append(...n) { this.children.push(...n); }, replaceChildren(...n) { this.children = n; }, setAttribute(k, v) { this.attrs[k] = v; },
  addEventListener(k, f) { this.handlers[k] = f; }, focus() {} });
const els = {}; for (const id of ['chat', 'chat-button', 'chat-close', 'chat-new', 'chat-form', 'chat-input', 'chat-send', 'chat-log']) els['start-' + id] = node();
const q = id => els['start-' + id];
const messages = { 'startup.chat_working': 'working', 'startup.chat_last_output': 'last output {value1} ago', 'startup.chat_no_output': 'no output yet' };
const acMessage = (id, v = {}) => (messages[id] || id).replace('{value1}', v.value1);
const context = vm.createContext({ document: { createElement: node }, Date, acMessage, console });
const elapsed = fs.readFileSync('src/ui/i18n.js', 'utf8').split('\n').find(l => l.startsWith('const AC_ELAPSED='));
vm.runInContext(elapsed + source + ';this.chatUi=chatUi;', context);
const tick = () => new Promise(r => setTimeout(r, 0));
const now = Date.now();
(async () => {
  let current = { chat_revision: 'r', chat_job: { id: 'j1', kind: 'chat', state: 'running', started_ms: now - 5000, output: { bytes: 0, updated_ms: null }, progress: [] }, chat: null };
  const ui = context.chatUi({ q, request: async p => (p.kind === 'chat_history' ? { turns: [{ role: 'operator', text: 'Where?' }] } : { id: 'j1' }), state: () => current, notice() {} });
  ui.update(); await tick();
  const log = q('chat-log'), bubble = () => log.children.at(-1);
  assert.equal(log.children.length, 2, 'user turn and one working bubble');
  assert.equal(bubble().children[0].textContent.endsWith('no output yet'), true, bubble().children[0].textContent);
  // Tool lines and the last output age appear in the same bubble.
  current = { ...current, chat_job: { ...current.chat_job, output: { bytes: 90, updated_ms: now - 12000 }, progress: [{ kind: 'text', text: 'Reading the project' }, { kind: 'tool', text: 'Read src/main.rs' }] } };
  ui.update(); await tick();
  assert.equal(log.children.length, 2, 'a redraw adds no bubble');
  assert.match(bubble().children[0].textContent, /last output 1[2-3]s ago/);
  assert.deepEqual(bubble().children.slice(1).map(c => c.textContent), ['Reading the project', '▸ Read src/main.rs']);
  assert.ok(bubble().children[2].className.includes('tool'));
  // The 1 s timer changes only the status label, not the progress lines.
  const lines = bubble().children.slice(1); ui.tick();
  assert.deepEqual(bubble().children.slice(1), lines, 'the timer keeps the progress nodes');
  // A reader who scrolled up stays where they are; one at the bottom follows.
  log.scrollTop = 100; ui.update(); await tick();
  assert.equal(log.scrollTop, 100, 'scrolled-up position is kept');
  log.scrollTop = 790; ui.update(); await tick();
  assert.equal(log.scrollTop, log.scrollHeight, 'a reader at the bottom follows');
  // The answer replaces the bubble in the same redraw.
  current = { ...current, chat_job: { id: 'j1', kind: 'chat', state: 'completed' }, chat: { job: 'j1', answer: 'In dist/.' } };
  ui.update(); await tick();
  assert.equal(log.children.length, 2, 'user turn and the answer');
  assert.ok(!log.children.some(c => c.className.includes('working')), 'no leftover bubble');
  console.log('The working bubble shows the CLI progress and output age without moving the reader.');
})().catch(e => { console.error(e); process.exit(1); });
