// GitHub #103: agent and model controls work for a task that has not started, even while others run.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
const start = source.indexOf('function canEditTask(');
assert.ok(start >= 0, 'room.js defines canEditTask');
const gate = (state, status, job) => {
  const context = vm.createContext({ snapshot: { report: { state }, job } });
  vm.runInContext("function busy(){return snapshot?.job?.state==='running';} function idle(){return snapshot&&!busy()&&!['running','completed'].includes(snapshot.report.state);}" + source.slice(start, source.indexOf('\n', start)), context);
  return vm.runInContext(`canEditTask(${JSON.stringify({ status })})`, context);
};
assert.equal(gate('running', 'pending'), true, 'a waiting task can be edited during a run');
assert.equal(gate('running', 'running'), false, 'a running task cannot');
assert.equal(gate('running', 'completed'), false, 'a finished task cannot');
assert.equal(gate('running', 'pending', { state: 'running' }), false, 'a busy UI job blocks it');
assert.equal(gate('stopped', 'running'), true, 'a stopped run keeps its existing rules');
assert.equal(gate('completed', 'pending'), false, 'a merged run is immutable');
assert.ok(source.includes("$('set-model').disabled=!canEditTask(t)||!models.length;$('reassign').disabled=!canEditTask(t);"), 'both buttons use the gate');
console.log('edit gate ok');
