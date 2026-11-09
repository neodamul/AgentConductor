// GitHub #637: a coordinator review that ended blocked without revisions offers Retry (a Resume) and Reassign an agent.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('blocked review notice skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-blocked-review-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const preset="SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.state='blocked';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.latest_review={verdict:'blocked',summary:'The evidence is ambiguous.',evidence:[],revisions:[]};";
(async()=>{try{const browser=await launch(chrome,{width:1280,height:900});try{
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href+'?run=blocked');
 await browser.eval("window.__beforePreset=true");
 await browser.eval(`PRESET(${JSON.stringify(preset)})`);
 let ready=false;for(let i=0;i<60;i++){try{ready=await browser.eval("!window.__beforePreset && !!document.getElementById('start-tab-tasks')");}catch{}if(ready)break;await new Promise(r=>setTimeout(r,50));}
 const result=await browser.eval(`(async()=>{
  const until=Date.now()+5000;while(document.getElementById('start-tab-tasks').disabled&&Date.now()<until)await new Promise(r=>setTimeout(r,30));
  document.getElementById('start-tab-tasks').click();
  const end=Date.now()+8000;while((document.getElementById('task-notice')||{hidden:true}).hidden&&Date.now()<end)await new Promise(r=>setTimeout(r,30));
  const box=document.getElementById('task-notice');
  if(box.hidden)throw Error('notice is shown');
  const buttons=[...box.querySelectorAll('button')].map(b=>b.textContent.trim());
  if(buttons.length!==2)throw Error('two choices: '+buttons);
  if(!box.textContent.includes('The evidence is ambiguous.'))throw Error('summary shown');
document.getElementById('task-notice').querySelectorAll('button')[1].click();
  const sel=document.getElementById('agent-select');if(document.activeElement!==(window.acChoices&&acChoices.buttonFor(sel)||sel))throw Error('Reassign focuses the agent choice');
  box.querySelectorAll('button')[0].click();
  const sent=Date.now()+5000;while(!SIM.log.some(e=>e.kind==='action'&&e.p.action.kind==='resume')&&Date.now()<sent)await new Promise(r=>setTimeout(r,30));
  if(!SIM.log.some(e=>e.kind==='action'&&e.p.action.kind==='resume'))throw Error('Retry sends resume');
  return buttons;
 })()`);
 console.log('blocked review notice ok',result.join('/'));
}finally{await browser.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
