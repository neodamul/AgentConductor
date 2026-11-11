// #841: failed startup must release only its owned process and temporary profile.
const assert = require('node:assert/strict'), fs = require('node:fs'), vm = require('node:vm');
const source = fs.readFileSync('tests/design_parity/cdp.cjs', 'utf8');
async function check(mode) {
  const removed = [], stopped = [], handlers = {}, owned = '/tmp/ac-owned-profile';
  let closed = false;
  const proc = {pid: mode === 'spawn' ? undefined : 1234, exitCode: mode === 'exit' ? 2 : null,
    signalCode: null, once(event, fn) {handlers[event] = fn;}, kill() {stopped.push(this.pid);}};
  const fakeFs = {mkdtempSync: () => owned, existsSync: () => ['discovery', 'socket'].includes(mode),
    readFileSync: () => '9222\n', rmSync: (dir, opts) => removed.push({dir, opts})};
  const module = {exports: {}};
  class Socket {constructor() {queueMicrotask(() => this.onerror(Error('socket failed')));} close() {closed = true;}}
  vm.runInNewContext(source, {module, require(name) {
    if (name === 'fs') return fakeFs;
    if (name === 'child_process') return {spawn() {if (mode === 'spawn') queueMicrotask(() => handlers.error?.(Error('ENOENT'))); return proc;}};
    if (name === './chrome_process.cjs') return {stopChrome: async p => stopped.push(p.pid)};
    return require(name);
  }, WebSocket: Socket, fetch: async () => {if (mode === 'discovery') throw Error('discovery failed'); return {json: async () => [{type: 'page', webSocketDebuggerUrl: 'ws://local'}]};},
  setTimeout: fn => {queueMicrotask(fn);return 1;}, clearTimeout() {}, process});
  const expected = {timeout: /Chrome did not start/, spawn: /ENOENT/, exit: /Chrome exited/, discovery: /discovery failed/, socket: /socket failed/};
  await assert.rejects(module.exports.launch('fake-chrome'), expected[mode]);
  assert.equal(stopped.length, mode === 'spawn' ? 0 : 1, mode + ': release only the spawned process');
  assert.equal(removed.length, 1, mode + ': remove the temporary profile');
  assert.equal(removed[0].dir, owned);
  assert.equal(removed[0].opts.recursive, true);
  if (mode === 'socket') assert.equal(closed, true);
}
(async () => {for (const mode of ['timeout', 'spawn', 'exit', 'discovery', 'socket']) await check(mode);
  console.log('5 Chrome startup failure cleanup scenarios passed');
})().catch(e => {console.error(e);process.exitCode = 1;});
