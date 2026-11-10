// GitHub #109: the selected session follows the run when the one being watched finishes.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
const start = source.indexOf('function followTarget(');
assert.ok(start >= 0, 'room.js defines followTarget');
const context = vm.createContext({});
vm.runInContext(source.slice(start, source.indexOf('\n', start)), context);
const snap = (state, statuses) => ({ report: { state, tasks: statuses.map(([id, status]) => ({ id, status })) } });
const follow = (prev, next, id, onCoordinator = false) => JSON.parse(vm.runInContext(`JSON.stringify(followTarget(${JSON.stringify(prev)}, ${JSON.stringify(next)}, ${JSON.stringify(id)}, ${onCoordinator}))`, context));
const a = [['a', 'running'], ['b', 'pending']];
assert.deepEqual(follow(snap('running', a), snap('running', [['a', 'completed'], ['b', 'running']]), 'a'), { id: 'b' }, 'moves to the next running session');
assert.deepEqual(follow(snap('running', a), snap('waiting_approval', [['a', 'completed'], ['b', 'completed']]), 'a'), { coordinator: true }, 'moves to the integration entry when the run awaits approval');
assert.equal(follow(snap('running', a), snap('running', a), 'a'), null, 'stays while the watched session still runs');
assert.equal(follow(snap('running', [['a', 'completed'], ['b', 'running']]), snap('running', [['a', 'completed'], ['b', 'running']]), 'a'), null, 'a reviewed finished session is not taken away');
assert.equal(follow(snap('running', a), snap('running', [['a', 'completed'], ['b', 'pending']]), 'a'), null, 'no running session yet, stay');
assert.equal(follow(snap('waiting_approval', [['a', 'completed']]), snap('approved', [['a', 'completed']]), 'a', true), null, 'integration view is not moved');
assert.equal(follow(null, snap('running', a), 'a'), null, 'first snapshot never moves');
assert.deepEqual(follow(snap('running', a), snap('running', [['a', 'failed'], ['b', 'running']]), 'a'), { id: 'b' }, 'a failed session also hands over');
// GitHub #688: a run that starts waiting for approval opens Integration verification even if the watched task was not the one that ran last.
assert.deepEqual(follow(snap('running', [['a', 'completed'], ['b', 'running']]), snap('waiting_approval', [['a', 'completed'], ['b', 'completed']]), 'a'), { coordinator: true }, 'a finished task being reviewed does not keep the rail');
assert.deepEqual(follow(null, snap('waiting_approval', [['a', 'completed'], ['b', 'completed']]), 'a'), { coordinator: true }, 'opening a run that already waits for approval lands on the integration entry');
assert.equal(follow(snap('waiting_approval', [['a', 'completed']]), snap('waiting_approval', [['a', 'completed']]), 'a'), null, 'the operator stays on the task they picked afterwards');
console.log('follow focus ok');
