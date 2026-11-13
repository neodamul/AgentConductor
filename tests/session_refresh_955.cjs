// #955: session-changing snapshots request one inventory; unchanged snapshots request none.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const nodes=new Map(),empty=()=>({dataset:{},hidden:true,open:false,addEventListener(){},setAttribute(){},replaceChildren(){},append(){},focus(){}});
const calls=[],waiters=[];let delayed=false;
const context={window:{},document:{getElementById:id=>{if(!nodes.has(id))nodes.set(id,empty());return nodes.get(id);},querySelectorAll:()=>[],createElement:empty},acMessage:key=>key,setTimeout:()=>0,clearTimeout(){}};
vm.createContext(context);vm.runInContext(fs.readFileSync('src/ui/agent_sessions.js','utf8'),context);
const sessions=context.window.acSessions;sessions.chip('codex',p=>{calls.push(p.kind);return delayed?new Promise(resolve=>waiters.push(resolve)):Promise.resolve({codex:[]});},id=>id);
const tick=()=>new Promise(resolve=>setImmediate(resolve));
const snapshot=(status,started=null)=>({run_dir:'/w/run-first',report:{state:'running',plan:{tasks:[{id:'t1',agent:'codex'}]},tasks:[{id:'t1',status,started_ms:started,finished_ms:null}]}});
(async()=>{
 assert.equal(typeof sessions.observe,'function','shared snapshot observer exists');
 sessions.observe(snapshot('pending'));await tick();assert.equal(calls.length,1);
 sessions.observe(snapshot('pending'));await tick();assert.equal(calls.length,1,'unchanged snapshot is free');
 sessions.observe(snapshot('running'));await tick();assert.equal(calls.length,2,'task state changed');
 sessions.observe(snapshot('running',100));await tick();assert.equal(calls.length,3,'session appears after status was already running');
 const irrelevant=snapshot('running',100);irrelevant.report.tests=[{duration_ms:99}];sessions.observe(irrelevant);await tick();assert.equal(calls.length,3,'test output does not trigger an inventory scan');
 delayed=true;sessions.observe(snapshot('completed',100));assert.equal(calls.length,4);
 const done=snapshot('completed',100);done.report.state='completed';sessions.observe(done);sessions.observe(done);assert.equal(calls.length,4,'coalesce changes during a scan');
 waiters.shift()({codex:[]});await tick();assert.equal(calls.length,5,'one final scan sees the latest state');waiters.shift()({codex:[]});await tick();delayed=false;
 sessions.observe({root:'/w',run_state:'running',session_revision:'a'});await tick();assert.equal(calls.length,6);
 sessions.observe({root:'/w',run_state:'running',session_revision:'a'});await tick();assert.equal(calls.length,6);
 sessions.observe({root:'/w',run_state:'running',session_revision:'b'});await tick();assert.equal(calls.length,7,'Workspace task revision changed');
 sessions.observe({root:'/other',run_state:'running',session_revision:'b'});await tick();assert.equal(calls.length,8,'workspace switches invalidate the cached session state');
 for(const [file,call] of [['src/ui/startup.js','acSessions.observe(state)'],['src/ui/room.js','window.acSessions?.observe(snapshot)']])assert(fs.readFileSync(file,'utf8').includes(call),file+' feeds the shared observer');
 console.log('session refresh #955: changed/unchanged, late start, bounded scans and both views passed');
})().catch(e=>{console.error(e);process.exitCode=1;});
