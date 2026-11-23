// GitHub #1085: a multi-phase Goal whose Phase 1 run completed stays writable for the next Phase's plan;
// only the active Goal's progress marks it complete, and the run report decides only without progress.
const fs=require('fs'),vm=require('vm'),assert=require('assert');
const ctx={acMessage:k=>k};vm.createContext(ctx);vm.runInContext(fs.readFileSync(__dirname+'/../src/ui/goal_readonly.js','utf8'),ctx);
const g=ctx.globalThis?.acGoalReadOnly||vm.runInContext('acGoalReadOnly',ctx),start={kind:'start'};
const phase=stage=>({run_state:'completed',slugs:[{slug:'g',active:true,progress:{stage,current_phase:'02-red'}}]});
assert.equal(g.blocks(phase('phase'),start),false,'a finished Phase run must not lock the next Phase');
assert.equal(g.blocks(phase('plan'),start),false,'planning the next Phase stays writable');
assert.equal(g.blocks(phase('completed'),start),true,'a completed Goal is read-only');
assert.equal(g.blocks({run_state:'completed',slugs:[{slug:'g',active:true}]},start),true,'without progress the run report decides');
assert.equal(g.blocks({report:{state:'completed'}},start),true,'the room report still marks a completed Goal');
assert.equal(g.blocks({run_state:'running',slugs:[]},start),false,'a running Goal stays writable');
console.log('multi-phase Goal stays writable until its progress completes');
