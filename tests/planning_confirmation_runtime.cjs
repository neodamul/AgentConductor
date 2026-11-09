const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const source=fs.readFileSync('src/ui/startup.js','utf8'),nodes=new Map();
function q(id){if(!nodes.has(id))nodes.set(id,{value:'',dataset:{},open:false,showModal(){this.open=true;},close(value){this.returnValue=value;this.open=false;this.onclose?.();}});return nodes.get(id);}
const errors=[],context={q,notice:(text,error)=>errors.push({text,error}),acMessage:(id,vars)=>JSON.stringify({id,...vars}),acAgentName:profile=>profile.id,AC_I18N:{bind(node,value){node.textContent=typeof value==='function'?value():value;}},state:{revision:'r1'},workspaceEpoch:0,draft:{goal:'Goal A',coordinator:'custom-cli',repository:'/project',profiles:[{id:'custom-cli',coordinator_model:'local-v1',catalog:{models:['local-v1']}}]},workspaceReviewKey:JSON.stringify};
vm.createContext(context);
const begin=source.indexOf('function planningContextKey('),end=source.indexOf('// DD-46:',begin);
assert(begin>=0&&end>begin,'explicit Goal planning confirmation exists');vm.runInContext(source.slice(begin,end),context);
const deadline=setTimeout(()=>{console.error('Planning confirmation regression timed out');process.exit(1);},3000);
(async()=>{
 let pending=vm.runInContext('confirmPlanning()',context);assert(q('planning-confirm').open);assert(q('planning-evidence').textContent.includes('Goal A'));assert(q('planning-evidence').textContent.includes('custom-cli'));assert(q('planning-evidence').textContent.includes('local-v1'));
 q('planning-confirm').close('cancel');assert.equal(await pending,null);
 pending=vm.runInContext('confirmPlanning()',context);q('planning-confirm').close('confirm');const selection=await pending;assert(selection);assert(context.planningUnchanged(selection));
 context.draft.profiles[0].coordinator_model='other';assert(!context.planningUnchanged(selection),'model change invalidates confirmation');assert.equal(await vm.runInContext('confirmPlanning()',context),null,'unknown model is rejected, never CLI default');
 context.draft.profiles[0].coordinator_model='';assert.equal(await vm.runInContext('confirmPlanning()',context),null,'empty explicit model is invalid, not CLI default');context.draft.profiles[0].coordinator_model='local-v1';pending=vm.runInContext('confirmPlanning()',context);context.state.revision='r2';q('planning-confirm').close('confirm');assert.equal(await pending,null,'external revision change needs review');
 pending=vm.runInContext('confirmPlanning()',context);context.workspaceEpoch++;q('planning-confirm').close('confirm');assert.equal(await pending,null,'Workspace change needs review');
 delete context.draft.profiles[0].coordinator_model;pending=vm.runInContext('confirmPlanning()',context);assert(q('planning-evidence').textContent.includes('startup.cli_default'));q('planning-confirm').oncancel();q('planning-confirm').close('cancel');assert.equal(await pending,null,'Escape cancels');
 context.draft.coordinator='missing';assert.equal(await vm.runInContext('confirmPlanning()',context),null,'missing coordinator cannot plan');
 const reviewLine=source.split('\n').find(line=>line.includes('function workspaceReviewKey('));vm.runInContext(reviewLine,context);context.draft.coordinator='custom-cli';const before=context.workspaceReviewKey(context.draft);context.draft.profiles[0].coordinator_model='local-v1';assert.notEqual(context.workspaceReviewKey(context.draft),before,'review key binds coordinator model');
 console.log('Goal planning confirmation: evidence, cancel/Escape, default, model, revision and Workspace guards pass');
})().catch(error=>{console.error(error);process.exitCode=1;}).finally(()=>clearTimeout(deadline));
