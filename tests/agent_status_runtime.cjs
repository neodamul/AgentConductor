// GitHub #410: the agent avatar state is the strongest state of the agent's tasks (error > waiting > running > completed).
const assert = require('node:assert/strict'), fs = require('node:fs'), vm = require('node:vm');
const cards = ['claude', 'codex', 'aipro'].map(id => { const symbol = { dataset: {} }; return { id, symbol, querySelector(sel) { return sel === '.agent-symbol' ? symbol : { textContent: id }; } }; });
const context = vm.createContext({ window: {}, document: { getElementById: () => null, querySelectorAll: () => cards }, MutationObserver: class { observe() {} } });
vm.runInContext(fs.readFileSync('src/ui/agent_status.js', 'utf8'), context);
const tone = s => ({ completed: 'ok', failed: 'bad', running: 'run', pending: 'idle', blocked_on_human: 'wait' }[s] || 'wait');
const state = id => cards.find(c => c.id === id).symbol.dataset;
const apply = (tasks, plan) => context.window.acAgentStatus.apply(tasks, plan, tone);

apply([{ status: 'running' }, { status: 'completed' }, { status: 'failed' }, { status: 'completed' }], [{ agent: 'claude' }, { agent: 'claude' }, { agent: 'codex' }, { agent: 'codex' }]);
assert.equal(state('claude').state, 'running'); assert.equal(state('claude').attention, undefined);
assert.equal(state('codex').state, 'error'); assert.equal(state('codex').attention, 'true');
assert.equal(state('aipro').state, undefined, 'an agent without tasks keeps the neutral border');

apply([{ status: 'completed' }, { status: 'pending' }], [{ agent: 'claude' }, { agent: 'claude' }]);
assert.equal(state('claude').state, undefined, 'an idle task keeps the agent from reading as done');
apply([{ status: 'completed' }], [{ agent: 'claude' }]);
assert.equal(state('claude').state, 'completed');
apply([{ status: 'blocked_on_human' }, { status: 'running' }], [{ agent: 'aipro' }, { agent: 'aipro' }]);
assert.equal(state('aipro').state, 'waiting'); assert.equal(state('aipro').attention, 'true');
console.log('Agent status: priority, neutral and attention markers passed');
