const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const source=fs.readFileSync('src/ui/startup.js','utf8'),html=fs.readFileSync('src/ui/startup.html','utf8'),nodes=new Map();
class Node {constructor(){this.hidden=true;this.className='';this.textContent='';this.attrs={};}setAttribute(k,v){this.attrs[k]=String(v);}}
const q=id=>{if(!nodes.has(id))nodes.set(id,new Node());return nodes.get(id);};
let finishAction,requested;
const context={q,state:{proposal:{plan:{tasks:[]},digest:'digest'},job:null,run_available:false},home:true,autoOpen:false,PLAN_START_LIMIT:300000,busy:()=>false,notice(){},poll:async()=>{},startText:()=>'Starting agent execution…',staleTail:()=>'',action:(packet)=>{requested=packet;return new Promise(resolve=>finishAction=resolve);},planActivity(text,stale,target){const box=q(target);box.hidden=false;box.className='plan-progress running';box.textContent=text();}};
vm.createContext(context);
vm.runInContext('let startPending=false,planStarting=0,clearedPlanJob=null,startClicked=0,clearedStartJob=null;',context);
const start=source.indexOf('function progressBoxes(){'),progress=source.slice(start,source.indexOf('\n  // Planning and starting',start));
vm.runInContext(progress,context);
context.planProgress=()=>vm.runInContext('progressBoxes()',context);
const handler=source.split('\n').find(line=>line.includes("q('run').onclick="));
assert(handler,'run button handler exists');vm.runInContext(handler,context);
assert.match(html,/<div id="start-run-progress"[^>]*role="status" aria-live="polite" hidden>/,'render accessible run-start status below the run button');
const pending=q('run').onclick();
assert.equal(requested.kind,'start','run button starts the approved plan directly');
assert.equal(requested.digest,'digest','send the exact reviewed plan digest');
assert.equal(q('run-progress').hidden,false,'show progress while the start request is pending');
assert.equal(q('run-progress').attrs['aria-busy'],'true','mark pending start feedback busy');
assert.match(q('run-progress').textContent,/Starting agent execution/,'show immediate start status');
finishAction(null);pending.then(()=>{
 assert.equal(q('run-progress').hidden,true,'clear progress after a rejected start');
 console.log('Direct run start shows accessible progress without a confirmation dialog.');
}).catch(error=>{console.error(error);process.exitCode=1;});
