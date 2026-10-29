// GitHub #115 / #185: the chat panel restores the stored conversation, sends the question, shows the answer once, keeps typed input and reports failures.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/chat.js', 'utf8');
const node = () => ({ children: [], hidden: false, disabled: false, value: '', textContent: '', className: '', scrollTop: 0, scrollHeight: 0, attrs: {}, handlers: {},
  append(...n) { this.children.push(...n); }, replaceChildren(...n) { this.children = n; }, setAttribute(k, v) { this.attrs[k] = v; },
  addEventListener(k, f) { this.handlers[k] = f; }, focus() {} });
const make = () => {
  const els = {}; // Like startup.js, q(id) looks up the element with id start-<id>.
  for (const id of ['chat', 'chat-button', 'chat-close', 'chat-new', 'chat-form', 'chat-input', 'chat-send', 'chat-log']) els['start-' + id] = node();
  els['start-chat'].hidden = true;
  // Strict: a missing element throws, as null.onclick does in the page.
  return id => { if (!els['start-' + id]) throw new Error('no element start-' + id); return els['start-' + id]; };
};
const tick = () => new Promise(r => setTimeout(r, 0));
const context = vm.createContext({ document: { createElement: node }, Date, acMessage: (id, v = {}) => id + (v.value1 ? ':' + v.value1 : ''), console });
vm.runInContext(source + ';this.chatUi=chatUi;', context);
const plain = v => JSON.parse(JSON.stringify(v));
const key = (q, init) => q('chat-input').handlers.keydown({ preventDefault() {}, isComposing: false, shiftKey: false, altKey: false, ...init });
(async () => {
  const calls = [], notices = []; let current = { revision: 'rev', chat_revision: 'chat-rev', job: null, chat: null }, stored = [], reply = { id: 'job-1' };
  const q = make();
  const ui = context.chatUi({ q, request: async p => { calls.push(p); if (p.kind === 'chat_history') return { turns: stored }; if (p.kind === 'chat_reset') return { turns: [] }; if (reply instanceof Error) throw reply; return reply; }, state: () => current, notice: (m, e) => notices.push([m, e]) });
  stored = [{ role: 'operator', text: 'What was built?' }, { role: 'assistant', text: 'A ladder game.' }];
  ui.update(); await tick();
  assert.deepEqual(plain(ui.turns()), stored, 'the stored conversation is shown again');
  assert.equal(q('chat-log').children.length, 2);
  q('chat-button').onclick();
  assert.equal(q('chat').hidden, false, 'the button opens the panel');
  q('chat-input').value = 'How do I run it?\nthen test';
  key(q, { key: 'Enter', shiftKey: true }); await tick();
  assert.equal(calls.filter(c => c.kind === 'chat').length, 0, 'Shift+Enter only breaks the line');
  key(q, { key: 'Enter' }); await tick();
  assert.deepEqual(plain(calls.at(-1)), { kind: 'chat', revision: 'chat-rev', text: 'How do I run it?\nthen test' });
  assert.equal(q('chat-send').disabled, true, 'sending is blocked while the coordinator works');
  assert.equal(q('chat-input').value, '', 'the input is cleared');
  current = { revision: 'rev', job: { id: 'job-1', kind: 'chat', state: 'completed' }, chat: { job: 'job-1', answer: 'Run:\n```\nnpm start\n```\nDone.' } };
  ui.update();
  assert.equal(ui.turns().length, 4);
  const answer = q('chat-log').children.at(-1).children;
  assert.ok(answer.some(c => c.className === 'chat-code' && c.textContent === 'npm start'), 'code fences keep their layout');
  ui.update();
  assert.equal(ui.turns().length, 4, 'an answer is shown once');
  // A failed job becomes an error turn; the next question carries only its own text.
  reply = { id: 'job-2' }; q('chat-input').value = 'And tests?'; key(q, { key: 'Enter' }); await tick();
  current = { revision: 'rev', chat_revision: 'chat-rev', job: { id: 'job-2', kind: 'chat', state: 'failed', error: 'no coordinator' }, chat: null };
  ui.update();
  assert.equal(ui.turns().at(-1).error, true);
  assert.ok(ui.turns().at(-1).text.includes('no coordinator'));
  // Up recalls earlier questions and Down returns to the draft.
  q('chat-input').value = ''; key(q, { key: 'ArrowUp' });
  assert.equal(q('chat-input').value, 'And tests?');
  key(q, { key: 'ArrowUp' });
  assert.equal(q('chat-input').value, 'How do I run it?\nthen test');
  key(q, { key: 'ArrowDown' }); key(q, { key: 'ArrowDown' });
  assert.equal(q('chat-input').value, '');
  // A refused request restores the typed text and reports the error.
  reply = new Error('another operation is running'); q('chat-input').value = 'Busy?'; key(q, { key: 'Enter' }); await tick();
  assert.equal(q('chat-input').value, 'Busy?');
  assert.equal(notices.at(-1)[0], 'another operation is running');
  // GitHub #197: a request that times out may still have started the job; the page asks the native side before rolling back.
  const savedHistory = stored; reply = new Error('timed out'); q('chat-input').value = 'Run the app';
  stored = [...savedHistory, { role: 'operator', text: 'Run the app' }];
  key(q, { key: 'Enter' }); await tick(); await tick();
  assert.equal(ui.turns().at(-1).text, 'Run the app', 'a question the native side accepted stays in the conversation');
  assert.equal(q('chat-input').value, '', 'and is not put back into the input');
  current = { revision: 'rev', chat_revision: 'chat-rev', job: { id: 'job-7', kind: 'chat', state: 'running' }, chat: null };
  ui.update();
  assert.equal(q('chat-send').disabled, true, 'the running chat is followed after the timeout');
  current = { revision: 'rev', chat_revision: 'chat-rev', job: { id: 'job-7', kind: 'chat', state: 'completed' }, chat: { job: 'job-7', answer: 'Started.' } };
  ui.update();
  assert.deepEqual(plain(ui.turns().at(-1)), { role: 'assistant', text: 'Started.' }, 'the late answer is shown');
  stored = savedHistory; current = { revision: 'rev', job: null, chat: null };
  reply = new Error('timed out'); q('chat-input').value = 'Never started'; key(q, { key: 'Enter' }); await tick(); await tick();
  assert.equal(q('chat-input').value, 'Never started', 'a question the native side never saw is restored');
  assert.ok(!ui.turns().some(t => t.text === 'Never started'));
  // A new conversation clears the screen and the stored history.
  q('chat-new').onclick(); await tick();
  assert.equal(calls.at(-1).kind, 'chat_reset');
  assert.equal(ui.turns().length, 0);
  q('chat-close').onclick();
  assert.equal(q('chat').hidden, true);
  // Reopening while the coordinator is still working picks the running chat up, then shows its answer.
  const q2 = make(); stored = [{ role: 'operator', text: 'Where are results?' }];
  current = { revision: 'rev', job: { id: 'job-9', kind: 'chat', state: 'running' }, chat: null };
  const ui2 = context.chatUi({ q: q2, request: async p => (p.kind === 'chat_history' ? { turns: stored } : { id: 'x' }), state: () => current, notice() {} });
  ui2.update(); await tick();
  assert.equal(q2('chat-send').disabled, true, 'a running chat is adopted');
  current = { revision: 'rev', job: { id: 'job-9', kind: 'chat', state: 'completed' }, chat: { job: 'job-9', answer: 'In dist/.' } };
  ui2.update();
  assert.deepEqual(plain(ui2.turns().at(-1)), { role: 'assistant', text: 'In dist/.' });
  assert.equal(q2('chat-send').disabled, false);
  // Another workspace starts with its own conversation.
  stored = []; ui2.reset(); assert.equal(ui2.turns().length, 0);
  console.log('chat runtime ok');
})().catch(e => { console.error(e); process.exitCode = 1; });
