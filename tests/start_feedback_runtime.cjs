const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const source=fs.readFileSync('src/ui/startup.js','utf8'),html=fs.readFileSync('src/ui/startup.html','utf8'),nodes=new Map();
class Node {constructor(){this.hidden=true;this.className="";this.textContent="";this.attrs={};this.dataset={revision:'rev',digest:'digest'};this.returnValue='confirm';this.childNodes=[];}setAttribute(k,v){this.attrs[k]=String(v);}addEventListener(_,fn){this.close=fn;}}
const q=id=>{if(!nodes.has(id))nodes.set(id,new Node());return nodes.get(id);};
let finishAction;
const context={q,state:{revision:'rev',config:{},job:null},draft:{},autoOpen:false,home:true,poll:async()=>{},PLAN_START_LIMIT:1,action:()=>new Promise(r=>finishAction=r),acMessage:()=> 'Starting agent execution…',startText:()=>'Starting agent execution…',staleTail:()=>'',AC_I18N:{replace(){}},notice(){},planActivity(text,stale,target='plan-progress'){const box=q(target);box.hidden=false;box.className="plan-progress running";box.textContent=text();}};
vm.createContext(context);
vm.runInContext('let startPending=false,planStarting=0,clearedPlanJob=null,startClicked=0,clearedStartJob=null,lastSnapshotAt=0;',context);
const progress=source.indexOf('function planProgress('),end=source.indexOf('// DD-46:',progress);
vm.runInContext(source.slice(progress,end),context);
const consent=source.split('\n').find(line=>line.includes("q('consent').addEventListener('close'"));
assert(consent,'consent start handler exists');vm.runInContext(consent,context);
const pending=q('consent').close();
assert.ok(/<div class="startup-review-footer">[\s\S]*?<\/div><div id="start-run-progress"[^>]*><\/div><\/section>/.test(html),'render the start status directly below the approve/run button');
assert.equal(q('run-progress').textContent,'Starting agent execution…','show status below the button before awaiting the start request');
assert.equal(q('run-progress').hidden,false);
assert.equal(q('run-progress').attrs['aria-busy'],'true');
finishAction(null);pending.then(()=>{assert.equal(q('run-progress').hidden,true);console.log('Approval shows accessible start feedback while pending and clears it after rejection.');});
