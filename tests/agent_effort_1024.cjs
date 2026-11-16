// GitHub #1024: the profile editor saves an agent's default model and effort, the models tag names them first, and
// the task inspector shows the agent default and sends a task's own effort (or clears it back to the default).
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('agent effort skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-agent-effort-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const preset="SIM.home.run_available=true;SIM.home.run_state='stopped';SIM.room.report.state='stopped';SIM.room.report.tasks.forEach(t=>t.status='pending');SIM.room.report.plan.tasks.forEach(t=>{t.agent='claude';});const c=SIM.room.report.plan.agents.find(a=>a.id==='claude');c.default_effort='high';SIM.home.config.profiles[0].catalog.models=['opus','sonnet'];";
(async()=>{try{const browser=await launch(chrome,{width:1280,height:900});try{
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href+'?run=effort');
 await browser.eval("window.__beforePreset=true");
 await browser.eval(`PRESET(${JSON.stringify(preset)})`);
 let ready=false;for(let i=0;i<60;i++){try{ready=await browser.eval("!window.__beforePreset && !!document.getElementById('start-tab-tasks')");}catch{}if(ready)break;await new Promise(r=>setTimeout(r,50));}
 const result=await browser.eval(`(async()=>{
  const wait=async(f,why)=>{const end=Date.now()+8000;while(!f()&&Date.now()<end)await new Promise(r=>setTimeout(r,30));if(!f())throw Error(why);};
  // Profile editor: pick a default model and effort, save, and the profile carries both.
  await wait(()=>document.querySelector('.startup-profile[data-agent="claude"]'),'agent card');
  document.querySelector('.startup-profile[data-agent="claude"] .profile-tools button').click();
  await wait(()=>document.getElementById('start-agent-dialog').open,'profile editor');
  const model=document.getElementById('start-default-model'),effort=document.getElementById('start-default-effort');
  if([...model.options].map(o=>o.value).join()!==',opus,sonnet')throw Error('default model lists the catalog: '+[...model.options].map(o=>o.value));
  if([...effort.options].map(o=>o.value).join()!==',low,medium,high,xhigh,max')throw Error('effort levels');
  model.value='sonnet';effort.value='xhigh';document.getElementById('start-agent-apply').click();
  await wait(()=>SIM.log.some(e=>e.kind==='save_profiles'),'profiles saved');
  const saved=SIM.log.filter(e=>e.kind==='save_profiles').at(-1).p.profiles.find(p=>p.id==='claude');
  document.getElementById('start-agent-dialog').close();
  if(saved.default_model!=='sonnet'||saved.default_effort!=='xhigh')throw Error('saved defaults: '+JSON.stringify(saved));
  // Task inspector: the agent default is named, a pick is sent, and the empty choice clears it.
  await wait(()=>!document.getElementById('start-tab-tasks').disabled,'tasks tab');document.getElementById('start-tab-tasks').click();
  const pick=document.getElementById('effort-select');
  await wait(()=>pick.options.length===6,'effort choices');
  if(!pick.options[0].textContent.includes('high'))throw Error('agent default named: '+pick.options[0].textContent);
  pick.value='low';document.getElementById('set-effort').click();
  await wait(()=>SIM.log.some(e=>e.kind==='action'&&e.p.action.kind==='effort'),'effort sent');
  const sent=SIM.log.filter(e=>e.kind==='action'&&e.p.action.kind==='effort').at(-1).p.action;
  if(sent.effort!=='low'||!sent.task)throw Error('effort action: '+JSON.stringify(sent));
  pick.value='';document.getElementById('set-effort').click();
  await wait(()=>SIM.log.filter(e=>e.kind==='action'&&e.p.action.kind==='effort').length===2,'clear sent');
  if(SIM.log.filter(e=>e.kind==='action'&&e.p.action.kind==='effort').at(-1).p.action.effort!==null)throw Error('empty choice clears');
  return 'ok';
 })()`);
 console.log('agent effort',result);
}finally{await browser.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
