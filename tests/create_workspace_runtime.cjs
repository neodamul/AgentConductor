// GitHub #511: "Create" in the New workspace dialog is not tied to the global `mutating` flag. Other work in progress never
// disables it; it waits for that work with a visible reason, and a finished action re-enables controls before its snapshot poll.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const source=fs.readFileSync('src/ui/startup.js','utf8');
const line=(start)=>{const a=source.indexOf(start);assert(a>=0,start);const b=source.indexOf('\n',a);return source.slice(a,b);};
assert(/q\('new-create'\)\.disabled=creating;/.test(source),'Create follows only its own flag');
assert(!/q\('new-create'\)\.disabled=mutating/.test(source),'Create is not coupled to unrelated work');
const nodes=new Map();
const q=id=>{if(!nodes.has(id))nodes.set(id,{value:'',disabled:false,text:'',close(){this.open=false;}});return nodes.get(id);};
const log=[];let release;const held=new Promise(r=>{release=r;});let pollDone;const pollGate=new Promise(r=>{pollDone=r;});
const context={q,log,setTimeout,Date,acMessage:key=>key,AC_I18N:{bind(node,fn){node.text=fn();}},PLAN_START_LIMIT:1,
  notice(){},reloadWorkspace:async()=>{log.push('reload');},firstWorkspaceNamed(){},
  request:async packet=>{log.push(packet.kind);if(packet.kind==='quota')await held;return packet.kind==='snapshot'?{job:null,metadata_job:null}:{};},
  poll:async()=>{log.push('poll');await pollGate;}};
vm.createContext(context);
vm.runInContext(`var mutating=false,maintenance=false,creating=false,state={revision:1};function renderButtons(){log.push('render:'+mutating);${"q('new-create').disabled=creating;"}}`,context);
vm.runInContext([line('  async function action('),line('  async function waitForWorkspaceIdle('),line('  async function createWorkspace(')].join('\n'),context);
const run=code=>vm.runInContext(code,context);
const until=async(ok,what)=>{for(let i=0;i<300;i++){if(ok())return;await new Promise(r=>setTimeout(r,10));}assert.fail('timed out: '+what);};
(async()=>{
  // 1. a quota refresh runs: Create stays enabled.
  const quota=run("action({kind:'quota'})");
  await until(()=>run('mutating'),'the action starts');
  assert.equal(run('mutating'),true);assert.equal(q('new-create').disabled,false,'Create is not disabled by a running action');
  // 2. Create is pressed meanwhile: it says why it waits and sends nothing yet.
  q('new-name').value='demo';
  const created=run('createWorkspace()');
  await until(()=>q('new-feedback').text==='startup.create_waits_for_running_work','the waiting reason is shown');
  await new Promise(r=>setTimeout(r,250));
  assert.equal(q('new-feedback').text,'startup.create_waits_for_running_work');
  assert(!log.includes('create_workspace'),'nothing is created while the other action runs');
  assert.equal(q('new-create').disabled,true,'a second press is ignored while this one waits');
  // 3. the action ends: the controls are re-enabled before its snapshot poll finishes.
  release();await until(()=>log.includes('poll')&&log.includes('create_workspace'),'the action polls and Create proceeds');
  assert(log.includes('poll'),'the action polls after the request');
  assert(log.indexOf('render:false')>=0&&log.indexOf('render:false')<log.indexOf('poll'),'controls are rendered before the follow-up poll');
  assert(log.includes('create_workspace'),'Create runs as soon as the other work is over, without waiting for the poll');
  pollDone();await quota;await created;
  assert.equal(q('new-create').disabled,false,'Create is enabled again');
  assert.equal(run('mutating'),false);assert.equal(run('creating'),false);
  console.log('Create is independent of other work, waits with a visible reason and re-enables after the action.');
})().catch(e=>{console.error(e);process.exit(1);});
