// #420: actual assembled UI consumes quota states and keeps chat/slide-over surfaces exclusive.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('Goal surfaces skipped: no Chrome');return;}const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-goal-surfaces-'));let b;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
for(const width of [840,1280]){b=await launch(chrome,{width});for(const theme of ['light','dark']){
 await b.goto(pathToFileURL(file).href);
 const result=await b.eval(`(async()=>{
 const q=id=>document.getElementById(id),checks=[],check=(name,ok)=>{if(!ok)throw Error(name);checks.push(name);};
 const until=async(f,name)=>{const end=Date.now()+15000;while(!f()){if(Date.now()>end)throw Error(name);await wait(20);}};
 const errors=[];addEventListener('error',e=>errors.push(e.message));addEventListener('unhandledrejection',e=>errors.push(String(e.reason)));document.documentElement.dataset.theme='${theme}';await until(()=>document.querySelector('#start-profiles .usage-button'),'profiles ready');
 const windowRow={kind:'window',index:0,label:'Five-hour fixture',remaining_percent:64,reset_label:'Resets Friday 12:50'};
 const states=[
  ['ok',{status:'available',rings:[windowRow],windows:[windowRow],budgets:[]}],
  ['unlimited',{status:'available',rings:[],windows:[],budgets:[{kind:'budget',index:0,label:'Unlimited fixture',unlimited:true}]}],
  ['unsupported',{status:'unsupported',rings:[],windows:[],budgets:[]}],
  ['error',{status:'unavailable',rings:[],windows:[],budgets:[],error:'Fixture quota unavailable'}],
 ];
 for(const [name,value]of states){SIM.home.quota_indicators.claude={source:'fixture-'+name,observed_at:Math.floor(Date.now()/1000),...value};document.dispatchEvent(new Event('visibilitychange'));
 await until(()=>document.querySelector('#start-profiles .usage-button').classList.contains('error')===(name==='error'),'state rendering');
 // Wait for actual snapshot consumption; a class shared by several states alone is insufficient.
 await until(()=>{const button=document.querySelector('#start-profiles .usage-button');button.click();const pop=q('usage-popover'),ready=pop?.textContent.includes('fixture-'+name);if(!ready)window.acQuotaRing.close(false);return ready;},'quota popup state '+name);
 const pop=q('usage-popover'),button=document.querySelector('#start-profiles .usage-button'),r=pop.getBoundingClientRect();
 check(name+' popup stays in viewport',r.left>=0&&r.top>=0&&r.right<=innerWidth&&r.bottom<=innerHeight);
 if(name==='ok')check('available quota/reset evidence',pop.textContent.includes('64%')&&pop.textContent.includes('Resets Friday 12:50')&&button.querySelector('.usage-value').getAttribute('stroke-dasharray')==='64 100');
 if(name==='unlimited')check('unlimited uses truthful label',pop.textContent.includes(acMessage('startup.quota_unlimited'))&&!pop.textContent.includes('100%'));
 if(name==='unsupported')check('unsupported has no invented rings',pop.textContent.includes(acMessage('startup.quota_unsupported'))&&!button.querySelector('.usage-value'));
 if(name==='error')check('error exposes actual failure',pop.querySelector('[role=alert]').textContent.includes('Fixture quota unavailable')&&!button.querySelector('.usage-value'));
 document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));check(name+' Escape restores opener',pop.hidden&&document.activeElement===button);
 }
 check('reading quota launches no provider request',!SIM.log.some(e=>e.kind==='quota'||e.kind==='prepare'||e.kind==='start'));
 q('start-chat-button').click();check('chat opens',!q('start-chat').hidden);q('start-goal-list-open').click();await until(()=>q('start-chat').hidden,'Goal closes chat');check('Goal only slide-over',document.querySelectorAll('dialog.slide-panel[open]').length===1);q('start-goals').close();
 q('start-chat-button').click();q('start-settings-button').click();await until(()=>q('start-chat').hidden,'Settings closes chat');check('Settings only slide-over',q('start-settings').open&&document.querySelectorAll('dialog.slide-panel[open]').length===1);q('start-settings').close();
 SIM.reply.workspace_briefing=()=>({state:'completed',observed_at_ms:Date.now(),sessions:[],approvals:[]});SIM.home.run_available=true;SIM.home.run_state='completed';document.dispatchEvent(new Event('visibilitychange'));await until(()=>!q('brief-team').disabled,'briefing enabled');q('start-chat-button').click();q('brief-team').click();await until(()=>q('briefing-dialog').open&&q('start-chat').hidden,'Briefing closes chat');check('Briefing only slide-over',document.querySelectorAll('dialog.slide-panel[open]').length===1);q('briefing-dialog').close();
 check('no UI runtime errors',errors.length===0);return checks;
 })()`);assert.equal(result.length,18,'all quota and surface assertions ran');console.log(theme,width,result.length,'quota states and exclusive surface checks passed');
}await b.close();b=null;}
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
