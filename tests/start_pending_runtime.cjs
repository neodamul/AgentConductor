// GitHub #269 (DD-207): the "starting" mark must end when the run opens, however late its report appears, or after a start that never opens a run; otherwise every button stays disabled until the app restarts.
const assert = require('node:assert/strict'), fs = require('node:fs'), vm = require('node:vm');
const source = fs.readFileSync('src/ui/startup.js', 'utf8');
const line = head => { const at = source.indexOf(head); assert.ok(at >= 0, head + ' exists in startup.js'); return source.slice(at, source.indexOf('\n', at)); };
let redraws = 0;
const context = vm.createContext({ request: async () => ({ opened: true, run: 'run-1' }), planProgress() { redraws++; }, window: { startRoom() {}, stopRoom() {} }, document: { body: { classList: { add() {} } }, getElementById: () => ({ hidden: true }) } });
vm.runInContext('let home=true,autoOpen=true,startPending=true;', context);
vm.runInContext(line('function startEnded(') + ';' + line('async function enter(') + ';this.enter=enter;this.startEnded=startEnded;', context);
const flags = () => JSON.parse(vm.runInContext('JSON.stringify({home,autoOpen,startPending})', context));
(async () => {
  // The start was accepted, the report was not there at the first poll, a later poll opens the run.
  assert.deepEqual(flags(), { home: true, autoOpen: true, startPending: true });
  await context.enter();
  assert.deepEqual(flags(), { home: false, autoOpen: false, startPending: false }, 'opening the run ends the starting mark');
  assert.ok(redraws >= 1, 'and the buttons are redrawn');
  // A start job that ends without opening a run releases the mark too.
  vm.runInContext('home=true;autoOpen=true;startPending=true;', context);
  context.startEnded();
  assert.equal(flags().startPending, false);
  console.log('start pending ok');
})().catch(e => { console.error(e); process.exitCode = 1; });
