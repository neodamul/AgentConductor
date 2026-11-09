// GitHub #410: the agent avatar state is the strongest state of the agent's tasks (error > waiting > running > completed).
const assert = require('node:assert/strict'), fs = require('node:fs'), vm = require('node:vm');
const cards = ['claude', 'codex', 'custom-cli'].map(id => { const symbol = { dataset: {} }; return { id, symbol, dataset: { agent: id }, querySelector(sel) { return sel === '.agent-symbol' ? symbol : null; } }; });
const context = vm.createContext({ window: {}, document: { getElementById: () => null, querySelectorAll: () => cards }, MutationObserver: class { observe() {} } });
vm.runInContext(fs.readFileSync('src/ui/agent_status.js', 'utf8'), context);
const tone = s => ({ completed: 'ok', failed: 'bad', running: 'run', pending: 'idle', blocked_on_human: 'wait' }[s] || 'wait');
const state = id => cards.find(c => c.id === id).symbol.dataset;
const apply = (tasks, plan) => context.window.acAgentStatus.apply(tasks, plan, tone);

apply([{ status: 'running' }, { status: 'completed' }, { status: 'failed' }, { status: 'completed' }], [{ agent: 'claude' }, { agent: 'claude' }, { agent: 'codex' }, { agent: 'codex' }]);
assert.equal(state('claude').state, 'running'); assert.equal(state('claude').attention, undefined);
assert.equal(state('codex').state, 'error'); assert.equal(state('codex').attention, 'true');
assert.equal(state('custom-cli').state, undefined, 'an agent without tasks keeps the neutral border');

apply([{ status: 'completed' }, { status: 'pending' }], [{ agent: 'claude' }, { agent: 'claude' }]);
assert.equal(state('claude').state, undefined, 'an idle task keeps the agent from reading as done');
apply([{ status: 'completed' }], [{ agent: 'claude' }]);
assert.equal(state('claude').state, 'completed');
apply([{ status: 'blocked_on_human' }, { status: 'running' }], [{ agent: 'custom-cli' }, { agent: 'custom-cli' }]);
assert.equal(state('custom-cli').state, 'waiting'); assert.equal(state('custom-cli').attention, 'true');
console.log('Agent status: priority, neutral and attention markers passed');
