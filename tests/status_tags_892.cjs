// GitHub #892: every status tag (heading pill, task outline badges, session states) uses the agent tag design.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('status tags skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-status-tags-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const preset="SIM.home.run_available=true;SIM.home.run_state='running';";
(async()=>{try{const browser=await launch(chrome,{width:1280,height:900});try{
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href);
 await browser.eval("window.__beforePreset=true");await browser.eval(`PRESET(${JSON.stringify(preset)})`);
 for(let i=0;i<60;i++){let r=false;try{r=await browser.eval("!window.__beforePreset&&!!document.getElementById('start-tab-tasks')");}catch{}if(r)break;await new Promise(r=>setTimeout(r,50));}
 const result=await browser.eval(`(async()=>{
  const wait=async(f,why)=>{const end=Date.now()+8000;while(!f()&&Date.now()<end)await new Promise(r=>setTimeout(r,30));if(!f())throw Error(why);};
  const tagged=t=>{const c=getComputedStyle(t);return c.borderTopWidth==='1px'&&c.borderTopStyle==='solid'&&c.borderTopLeftRadius==='4px';};
  await wait(()=>!document.getElementById('start-tab-tasks').disabled,'tasks tab');document.getElementById('start-tab-tasks').click();
  await wait(()=>document.querySelectorAll('.outline-task .outline-state-badge').length>=2,'outline badges');
  const badges=[...document.querySelectorAll('.outline-task .outline-state-badge')];if(!badges.every(tagged))throw Error('outline badges carry the tag border');
  const running=badges.find(b=>b.dataset.state==='running');if(!running)throw Error('a running badge');
  const pill=document.querySelector('.goal-info>.status-pill');if(pill&&pill.getClientRects().length&&!tagged(pill))throw Error('the heading pill carries the tag border');
  SIM.reply.agent_sessions=()=>({claude:[{run:'run-a',task:'t',label:'T1 t',goal:'g',state:'running',started_ms:1,last_ms:1,note:null,cleanable:false,stoppable:true}]});
  await acSessions.refresh();acSessions.open('claude');await wait(()=>document.querySelector('#start-agent-sessions-list .session-state.running'),'session state');
  const state=document.querySelector('#start-agent-sessions-list .session-state.running');if(!tagged(state))throw Error('session states carry the tag border');
  const tag=document.querySelector('[data-state-tag="running"]');if(tag&&getComputedStyle(tag).color!==getComputedStyle(state).color)throw Error('running uses one info color');
  if(getComputedStyle(running).color!==getComputedStyle(state).color)throw Error('outline running matches the info color');
  return badges.length+' badges';
 })()`);
 console.log('status tags ok',result);
}finally{await browser.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
