// GitHub #285: a request the operator started marks the pressed button and, after 1 s, is named with
// its time on the status line; refreshes stay silent. Both pages send every request through it.
const vm = require('node:vm'), fs = require('node:fs'), assert = require('node:assert/strict');
// Windows checkouts may have CRLF line ends.
const read = f => fs.readFileSync(f, 'utf8').replace(/\r\n/g, '\n');
const i18n = read('src/ui/i18n.js'), startup = read('src/ui/startup.js'), room = read('src/ui/room.js');
const catalog = JSON.parse(fs.readFileSync('src/ui/messages/en.json', 'utf8'));
assert.ok(catalog['common.waiting_reply'] && JSON.parse(fs.readFileSync('src/ui/messages/ko.json', 'utf8'))['common.waiting_reply'], 'the line is in both catalogs');
let clickHandler = null, now = 1_000_000;
const document = { addEventListener(kind, handler) { if (kind === 'click') clickHandler = handler; } };
const clock = { now: () => now };
const acMessage = (id, v = {}) => catalog[id].replace(/\{(\w+)\}/g, (_, k) => String(v[k]));
const ctx = vm.createContext({ document, Date: clock, Promise, acMessage });
assert.ok(i18n.includes('const AC_WAIT='), 'AC_WAIT exists');
const line = h => i18n.slice(i18n.indexOf(h), i18n.indexOf('\n', i18n.indexOf(h)) + 1);
vm.runInContext(line('const AC_ELAPSED=') + i18n.slice(i18n.indexOf('const AC_PRESS=')) + ';this.AC_WAIT=AC_WAIT;', ctx);
const button = () => { const classes = new Set(); return { dataset: {}, closest() { return this; }, getClientRects: () => [{}], classList: { toggle(c, on) { on ? classes.add(c) : classes.delete(c); }, contains: c => classes.has(c) }, setAttribute() {}, removeAttribute() {} }; };
(async () => {
  const test = button(); clickHandler({ target: test });
  let finish; const pending = ctx.AC_WAIT.watch('test_messenger', new Promise(r => { finish = r; }));
  assert.ok(test.classList.contains('is-busy'), 'the pressed button is marked');
  assert.equal(ctx.AC_WAIT.text(now + 900), '', 'nothing is said before 1 s');
  assert.equal(ctx.AC_WAIT.text(now + 4000), acMessage('common.waiting_reply', { value1: 'test messenger', value2: '0:04' }));
  finish('ok'); assert.equal(await pending, 'ok');
  assert.ok(!test.classList.contains('is-busy'));
  assert.equal(ctx.AC_WAIT.text(now + 5000), '', 'an answered request is no longer named');
  // A refresh never marks the last pressed button and is never named.
  const tab = button(); clickHandler({ target: tab });
  const poll = ctx.AC_WAIT.watch('snapshot', new Promise(() => {}));
  assert.ok(!tab.classList.contains('is-busy')); assert.equal(ctx.AC_WAIT.text(now + 9000), '');
  void poll;
  // A failed request is no longer named either, and its error reaches the caller.
  await assert.rejects(ctx.AC_WAIT.watch('delete_workspace', Promise.reject(Error('locked'))), /locked/);
  assert.equal(ctx.AC_WAIT.text(now + 9000), '');
  // Both pages route their requests through it and show the line.
  assert.match(startup, /async function request\(packet,limit=15000,watched\)\{[^\n]*if\(!watched&&typeof AC_WAIT!=='undefined'\)return AC_WAIT\.watch\(packet\.kind,request\(packet,limit,true\)\);/);
  assert.match(room, /async function request\(packet, watched\) \{\n[^\n]*if \(!watched && typeof AC_WAIT !== 'undefined'\) return AC_WAIT\.watch\(packet\.kind === 'action' \? packet\.action\.kind : packet\.kind, request\(packet, true\)\);/);
  assert.ok(startup.includes('AC_WAIT.text()') && room.includes('AC_WAIT.text(now)'), 'the status line and the room connection show it');
  assert.ok(room.includes("(typeof AC_WAIT!=='undefined'&&AC_WAIT.text())||acMessage(\"room.connected_local_run\")"), 'a refresh of the room keeps the waiting line instead of writing connected over it');
  console.log('request wait: pressed button, 1 s line, silent refreshes and both pages');
})().catch(e => { console.error(e); process.exitCode = 1; });
