// #1004: one click on Merge automatically sticks: polls during the save do not undo it, and a click while another save runs is queued, not dropped.
const fs=require('fs'),os=require('os'),path=require('path'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('Auto-merge click skipped: no Chrome');return;}const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-auto-merge-1004-'));let b;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(chrome,{width:1280,height:960});
 const setup=`SIM.home.proposal=SIM.proposal('ready');SIM.home.proposal.slug='g';SIM.home.proposal.slug_phase='p1';SIM.home.config.auto_merge=false;SIM.home.slugs=[{slug:'g',active:true,goal:SIM.home.config.goal,plan:{phases:[]},progress:{current_phase:'p1',phases:[1,2].map(n=>({id:'p'+n,name:'Phase '+n,stage:'plan',task:[],completed:[],verification:[]}))}}];SIM.reply.save_auto_merge=p=>{SIM.home.config.auto_merge=p.enabled;SIM.home.revision+='x';return {saved:true};};`;
 await b.goto(pathToFileURL(file).href);await b.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(setup)})`);await b.goto(pathToFileURL(file).href);
 await b.eval(`(async()=>{const end=Date.now()+6000;while(!document.querySelector('#start-tasks .startup-task')){if(Date.now()>end)throw Error('No plan');await wait(30);}document.getElementById('start-tab-review').click();})()`);
 await b.eval(`(async()=>{const box=document.getElementById('start-auto-merge'),saves=()=>SIM.log.filter(x=>x.kind==='save_auto_merge');
  SIM.hang.save_auto_merge=true;box.click();
  const end=Date.now()+4000;while(!saves().length){if(Date.now()>end)throw Error('no save request');await wait(30);}
  // Background polls still report the old setting while the save is held.
  await wait(2500);if(!box.checked)throw Error('a poll undid the click while saving');
  // A second click while the first save is still running is kept and saved after it.
  box.click();await wait(300);if(box.checked)throw Error('the second click was dropped');
  SIM.hang.save_auto_merge=false;SIM.waiters.save_auto_merge.splice(0).forEach(f=>f());
  const done=Date.now()+6000;while(saves().length<2){if(Date.now()>done)throw Error('queued click was not saved: '+saves().length);await wait(30);}
  await wait(1500);if(box.checked||SIM.home.config.auto_merge!==false)throw Error('final state does not match the last click: '+box.checked+' '+SIM.home.config.auto_merge);})()`);
 console.log('Auto-merge click sticks through polls and a queued second click');
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
