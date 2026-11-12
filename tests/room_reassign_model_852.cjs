// GitHub #852: picking another agent in the task inspector lists that agent's models, and Assign sends both.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('room reassign model skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-reassign-model-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const preset="SIM.home.run_available=true;SIM.home.run_state='stopped';SIM.room.report.state='stopped';SIM.room.report.tasks.forEach(t=>t.status='pending');SIM.room.report.plan.tasks.forEach(t=>{t.agent='claude';t.model='opus';});";
(async()=>{try{const browser=await launch(chrome,{width:1280,height:900});try{
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href+'?run=reassign');
 await browser.eval("window.__beforePreset=true");
 await browser.eval(`PRESET(${JSON.stringify(preset)})`);
 let ready=false;for(let i=0;i<60;i++){try{ready=await browser.eval("!window.__beforePreset && !!document.getElementById('start-tab-tasks')");}catch{}if(ready)break;await new Promise(r=>setTimeout(r,50));}
 const result=await browser.eval(`(async()=>{
  const wait=async(f,why)=>{const end=Date.now()+8000;while(!f()&&Date.now()<end)await new Promise(r=>setTimeout(r,30));if(!f())throw Error(why);};
  await wait(()=>!document.getElementById('start-tab-tasks').disabled,'tasks tab');document.getElementById('start-tab-tasks').click();
  const agent=document.getElementById('agent-select'),model=document.getElementById('model-select'),values=()=>[...model.options].map(o=>o.value).join();
  await wait(()=>agent.value==='claude'&&values()==='opus','claude task lists opus');
  agent.value='codex';agent.dispatchEvent(new Event('change',{bubbles:true}));
  await wait(()=>values()==='gpt','codex lists its own models');
  if(model.value!=='gpt')throw Error('codex model picked');
  if(!document.getElementById('set-model').disabled)throw Error('Set waits for Assign when the agent changed');
  if(agent.value!=='codex')throw Error('the picked agent survives a repaint');
  document.getElementById('reassign').click();
  await wait(()=>SIM.log.some(e=>e.kind==='action'&&e.p.action.kind==='reassign'),'reassign sent');
  const sent=SIM.log.find(e=>e.kind==='action'&&e.p.action.kind==='reassign').p.action;
  if(sent.agent!=='codex'||sent.model!=='gpt')throw Error('agent and model sent together: '+JSON.stringify(sent));
  return sent.agent+'/'+sent.model;
 })()`);
 console.log('room reassign model ok',result);
}finally{await browser.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
