// GitHub #95: Send feedback works for a task that is running; the backend queues it.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
const start = source.indexOf('function feedbackEnabled(');
assert.ok(start >= 0, 'room.js defines feedbackEnabled');
const end = source.indexOf('\n', start);
const gate = (state, status, extra = {}) => {
  const context = vm.createContext({
    snapshot: { report: { state }, job: extra.job },
    coordinatorView: extra.coordinatorView || false,
  });
  vm.runInContext("function busy() { return snapshot?.job?.state==='running'; }" + source.slice(start, end), context);
  return vm.runInContext(`feedbackEnabled(${JSON.stringify({ status })})`, context);
};
assert.equal(gate('running', 'running'), true, 'a running task accepts queued feedback');
assert.equal(gate('running', 'completed'), false, 'a finished task cannot be edited while others run');
assert.equal(gate('running', 'running', { coordinatorView: true }), false, 'the coordinator entry takes no feedback');
assert.equal(gate('running', 'running', { job: { state: 'running' } }), false, 'a busy UI job blocks it');
assert.equal(gate('waiting_approval', 'completed'), true, 'a stopped run still accepts feedback');
assert.equal(gate('completed', 'completed'), false, 'a merged run is immutable');
console.log('feedback gate ok');
