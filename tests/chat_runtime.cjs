// GitHub #115: the chat panel sends the conversation, shows the answer once the job completes, and reports failures.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/chat.js', 'utf8');
const node = () => ({ children: [], hidden: false, disabled: false, value: '', textContent: '', className: '', scrollTop: 0, scrollHeight: 0, attrs: {}, handlers: {},
  append(...n) { this.children.push(...n); }, replaceChildren(...n) { this.children = n; }, setAttribute(k, v) { this.attrs[k] = v; },
  addEventListener(k, f) { this.handlers[k] = f; }, focus() {} });
const els = {}; const q = id => els[id] ||= node();
for (const id of ['start-chat', 'start-chat-button', 'start-chat-close', 'start-chat-form', 'start-chat-input', 'start-chat-send', 'start-chat-log']) q(id);
els['start-chat'].hidden = true;
const submit = async () => { q('start-chat-form').onsubmit({ preventDefault() {} }); await new Promise(r => setTimeout(r, 0)); };
const calls = [], notices = []; let current = { revision: 'rev', job: null, chat: null };
const context = vm.createContext({ document: { createElement: node }, acMessage: (id, v = {}) => id + (v.value1 ? ':' + v.value1 : ''), console });
vm.runInContext(source + ';this.chatUi=chatUi;', context);
(async () => {
  let reply = { id: 'job-1' };
  const ui = context.chatUi({ q, request: async p => { calls.push(p); if (reply instanceof Error) throw reply; return reply; }, state: () => current, notice: (m, e) => notices.push([m, e]) });
  q('start-chat-button').onclick();
  assert.equal(q('start-chat').hidden, false, 'the button opens the panel');
  q('start-chat-input').value = 'How do I run it?';
  await submit();
  assert.equal(calls.length, 1);
  assert.deepEqual(JSON.parse(JSON.stringify(calls[0])), { kind: 'chat', revision: 'rev', turns: [{ role: 'operator', text: 'How do I run it?' }] });
  assert.equal(q('start-chat-send').disabled, true, 'sending is blocked while the coordinator works');
  assert.equal(q('start-chat-input').value, '', 'the input is cleared');
  current = { revision: 'rev', job: { id: 'job-1', state: 'completed' }, chat: { job: 'job-1', answer: 'Run npm start.' } };
  ui.update();
  assert.equal(ui.turns().length, 2);
  assert.deepEqual(JSON.parse(JSON.stringify(ui.turns()[1])), { role: 'assistant', text: 'Run npm start.' });
  assert.equal(q('start-chat-send').disabled, false);
  ui.update();
  assert.equal(ui.turns().length, 2, 'an answer is shown once');
  // A failed job becomes an error turn that is not sent back to the coordinator.
  reply = { id: 'job-2' }; q('start-chat-input').value = 'And tests?';
  await submit();
  current = { revision: 'rev', job: { id: 'job-2', state: 'failed', error: 'no coordinator' }, chat: null };
  ui.update();
  assert.equal(ui.turns().at(-1).error, true);
  assert.ok(ui.turns().at(-1).text.includes('no coordinator'));
  reply = { id: 'job-3' }; q('start-chat-input').value = 'Try again';
  await submit();
  const sent = JSON.parse(JSON.stringify(calls.at(-1).turns.map(t => t.text)));
  assert.deepEqual(sent, ['How do I run it?', 'Run npm start.', 'And tests?', 'Try again'], 'error turns are not part of the conversation');
  // A request that is refused restores the typed text and reports the error.
  current = { revision: 'rev', job: { id: 'job-3', state: 'completed' }, chat: { job: 'job-3', answer: 'ok' } }; ui.update();
  reply = new Error('another operation is running'); q('start-chat-input').value = 'Busy?';
  await submit();
  assert.equal(q('start-chat-input').value, 'Busy?');
  assert.equal(notices.at(-1)[0], 'another operation is running');
  q('start-chat-close').onclick();
  assert.equal(q('start-chat').hidden, true);
  console.log('chat runtime ok');
})().catch(e => { console.error(e); process.exitCode = 1; });
