const assert = require('node:assert/strict');
const fs = require('node:fs');

const listeners = new Map();
const notices = [];
global.document = {
  hidden: false,
  addEventListener: (name, fn) => listeners.set(name, fn),
  removeEventListener: (name) => listeners.delete(name),
};
global.window = {__acRequest: () => new Promise(() => {})};
global.acMessage = (key, values) => `${key} (${values.value1})`;

async function timeoutPausesWhileHidden(file, signature, endMarker, limit, includeEnd = 0) {
  const source = fs.readFileSync(file, 'utf8');
  const start = source.indexOf(signature);
  const end = source.indexOf(endMarker, start);
  assert.notEqual(start, -1, `${signature} exists`);
  const original = {setTimeout: global.setTimeout, clearTimeout: global.clearTimeout, performance: global.performance};
  let now = 0, nextId = 0;
  const timers = new Map();
  Object.defineProperty(global, 'performance', {configurable: true, value: {now: () => now}});
  global.setTimeout = (fn, delay) => { const id = ++nextId; timers.set(id, {fn, delay}); return id; };
  global.clearTimeout = id => timers.delete(id);
  global.request = new Function(`${source.slice(start, end + includeEnd)}; return request;`)();
  try {
    const pending = request({kind: 'snapshot'}, limit);
    const [id, initial] = [...timers.entries()][0];
    now += 10;
    document.hidden = true;
    const visibility = listeners.get('visibilitychange');
    assert.equal(typeof visibility, 'function', 'requests watch page visibility');
    visibility();
    assert.equal(timers.size, 0, 'hidden time is removed from the timeout');
    let settled = false;
    pending.catch(() => { settled = true; });
    now += limit * 10;
    assert.equal(settled, false, 'a hidden page does not consume the request timeout');
    document.hidden = false;
    visibility();
    const [resumedId, resumed] = [...timers.entries()][0];
    assert.ok(resumed.delay < initial.delay, 'the request resumes with only its remaining visible time');
    now += resumed.delay;
    timers.delete(resumedId);
    resumed.fn();
    await assert.rejects(pending, /startup\.native_response_timed_out/);
    assert.equal(listeners.has('visibilitychange'), false, 'settled requests remove their listener');
  } finally {
    document.hidden = false;
    global.setTimeout = original.setTimeout;
    global.clearTimeout = original.clearTimeout;
    Object.defineProperty(global, 'performance', {configurable: true, value: original.performance});
  }
}

async function lateTimeoutGetsANewDeadline(file, signature, endMarker, limit, includeEnd = 0) {
  const source = fs.readFileSync(file, 'utf8');
  const start = source.indexOf(signature);
  const end = source.indexOf(endMarker, start);
  assert.notEqual(start, -1, `${signature} exists`);
  const requestSource = source.slice(start, end + includeEnd);
  const original = {setTimeout: global.setTimeout, clearTimeout: global.clearTimeout, performance: global.performance};
  let now = 0, nextId = 0;
  const timers = new Map();
  Object.defineProperty(global, 'performance', {configurable: true, value: {now: () => now}});
  global.setTimeout = (fn, delay) => { const id = ++nextId; timers.set(id, {fn, delay}); return id; };
  global.clearTimeout = id => timers.delete(id);
  global.request = new Function(`${requestSource}; return request;`)();
  try {
    const pending = request({kind: 'snapshot'}, limit);
    const [id, first] = [...timers.entries()][0];
    now = limit + 6000;
    timers.delete(id);
    first.fn();
    await Promise.resolve();
    assert.equal(timers.size, 1, 'a late suspended-page timer restarts instead of timing out');
    const [nextId, next] = [...timers.entries()][0];
    now += next.delay;
    timers.delete(nextId);
    next.fn();
    await assert.rejects(pending, /startup\.native_response_timed_out/);
  } finally {
    global.setTimeout = original.setTimeout;
    global.clearTimeout = original.clearTimeout;
    Object.defineProperty(global, 'performance', {configurable: true, value: original.performance});
  }
}

async function backgroundPollTimeoutsAreSilent() {
  global.request = async () => { throw Error('Request timed out (snapshot)'); };
  global.notice = (...args) => notices.push(args);
  global.notices = notices;
  global.polling = false;
  global.home = true;
  global.workspaceEpoch = 0;
  global.draft = {profiles: []};
  global.state = {job: null};
  global.iconKinds = '';
  global.quotaRefreshed = new Map();
  global.sources = {wantsResult: () => false};
  global.mutating = false;
  global.planStarting = 0;
  global.PLAN_START_LIMIT = 15000;
  const startup = fs.readFileSync('src/ui/startup.js', 'utf8');
  assert.match(startup, /await poll\(true\);schedule\(\)/, 'workspace timer uses silent poll mode');
  assert.match(startup, /poll\(true\)\.then\(schedule\)/, 'workspace startup and visibility resume use silent poll mode');
  const start = startup.indexOf('async function poll(background=false)');
  const end = startup.indexOf("\n for(const id of ['name'", start);
  const poll = new Function(`${startup.slice(start, end)}; return poll;`)();
  await poll(true);
  assert.equal(notices.length, 0, 'scheduled workspace polling does not toast on timeout');
  await poll();
  assert.equal(notices.length, 1, 'an explicit workspace refresh still reports an error');

  notices.length = 0;
  global.polling = false;
  global.roomStopped = false;
  global.snapshot = {report: {plan: {agents: []}}};
  global.AC_I18N = {bind() {}};
  global.$ = () => ({});
  const room = fs.readFileSync('src/ui/room.js', 'utf8');
  assert.match(room, /await poll\(true\);roomSchedule\(\)/, 'room timer uses silent poll mode');
  assert.match(room, /poll\(true\)\.then\(roomSchedule\)/, 'room startup and visibility resume use silent poll mode');
  const roomStart = room.indexOf('async function poll(background=false)');
  const roomEnd = room.indexOf('\nfor(const el', roomStart);
  const roomPoll = new Function(`${room.slice(roomStart, roomEnd)}; return poll;`)();
  await roomPoll(true);
  assert.equal(notices.length, 0, 'scheduled room polling does not toast on timeout');
  await roomPoll();
  assert.equal(notices.length, 1, 'an explicit room refresh still reports an error');
}

async function main() {
  await timeoutPausesWhileHidden('src/ui/startup.js', 'async function request(packet,limit=15000){', '\n', 100);
  await timeoutPausesWhileHidden('src/ui/room.js', 'async function request(packet) {', '\n}\n', 15000, 2);
  await lateTimeoutGetsANewDeadline('src/ui/startup.js', 'async function request(packet,limit=15000){', '\n', 100);
  await lateTimeoutGetsANewDeadline('src/ui/room.js', 'async function request(packet) {', '\n}\n', 15000, 2);
  await backgroundPollTimeoutsAreSilent();
  console.log('idle poll timeout regression passed');
}

main().catch(error => { console.error(error); process.exitCode = 1; });
