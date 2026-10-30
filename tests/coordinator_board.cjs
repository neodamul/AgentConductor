// GitHub #267 (DD-206): the coordinator Activity tab starts with a status board and does not draw the workflow rows.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
const line = name => { const at = source.indexOf('function ' + name + '('); assert.ok(at >= 0, name + ' exists in room.js'); return source.slice(at, source.indexOf('\n', at)); };
const context = vm.createContext({});
vm.runInContext(line('shownHistory') + ';' + line('boardCounts') + ';this.shownHistory=shownHistory;this.boardCounts=boardCounts;', context);
const plain = v => JSON.parse(JSON.stringify(v));
const row = (seq, event, summary = 'x') => ({ seq, event, summary });
const rows = [row(1, 'workflow', 'run phase=running completed=0 failed=0'), row(5, 'availability_timer'), row(6, 'workflow', 'run phase=running completed=0 failed=0'), row(7, 'task_profile'), row(10, 'workflow', 'run phase=running completed=1 failed=0'), row(11, 'tool_orchestrator_decision')];
assert.deepEqual(plain(context.shownHistory(rows)).map(r => [r.seq, r.event]), [[5, 'availability_timer'], [7, 'task_profile'], [11, 'tool_orchestrator_decision']], 'workflow rows are not drawn; the informative rows stay in order');
assert.equal(rows.length, 6, 'the stored rows are not changed');
assert.deepEqual(plain(context.boardCounts([{ status: 'completed' }, { status: 'running' }, { status: 'running' }, { status: 'failed' }, { status: 'pending' }, { status: 'stopped' }])), { total: 6, completed: 1, running: 2, failed: 1, pending: 1 });
console.log('coordinator board ok');
