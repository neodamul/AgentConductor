// #420: joined assembled-UI Workspace -> Goal -> plan -> run -> results -> new Goal -> deletion.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs'),preset=require('./ui_sim/lifecycle.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('Goal lifecycle skipped: no Chrome');return;}const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-goal-lifecycle-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
for(const width of [840,1280]){browser=await launch(chrome,{width});for(const theme of ['light','dark']){
 await browser.goto(pathToFileURL(file).href);await browser.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(preset)})`);await browser.goto(pathToFileURL(file).href);
 const checked=await browser.eval(`(async()=>{
 const q=id=>document.getElementById(id),checks=[],check=(name,ok)=>{if(!ok)throw Error(name);checks.push(name);};
 const until=async(f,name)=>{const end=Date.now()+15000;while(!f()){if(Date.now()>end)throw Error(name+' '+JSON.stringify({logs:SIM.log.slice(-10).map(x=>x.kind),notice:q('start-notice').textContent,errors:flowErrors,open:[...document.querySelectorAll('dialog[open]')].map(d=>d.id)}));await wait(30);}};
 const input=(id,value)=>{q(id).value=value;q(id).dispatchEvent(new Event('input'));};
 document.documentElement.dataset.theme='${theme}';await until(()=>q('start-new-workspace').open,'first Workspace dialog');
 input('start-new-name','Lifecycle Workspace');q('start-new-create').click();await until(()=>!q('start-new-workspace').open&&!q('start-save').disabled&&q('start-name').value==='Lifecycle Workspace','created Workspace settles');
 check('create Workspace uses real UI request',SIM.log.some(e=>e.kind==='create_workspace'&&e.p.name==='Lifecycle Workspace'));
 input('start-goal','Implement the joined Goal');const inherited=JSON.stringify(SIM.home.config.roles);q('start-save').click();await until(()=>!q('start-panel-team').hidden&&!q('start-prepare').disabled,'saved Goal and Team');
 q('start-prepare').click();await until(()=>q('start-planning-confirm').open,'planner confirmation');q('start-planning-confirm').close('confirm');
 await until(()=>SIM.home.proposal?.phase==='ready'&&!q('start-run').disabled,'review ready');check('planning uses registered coordinator and model',SIM.home.config.coordinator==='claude'&&SIM.home.config.profiles[0].coordinator_model==='opus');
 q('start-run').click();await until(()=>q('start-consent').open,'exact plan consent');q('start-consent').close('confirm');
 await until(()=>document.body.classList.contains('on-run')&&document.querySelectorAll('.session-choice .task-id').length===3,'running Room');
 check('run starts only after exact consent',SIM.log.filter(e=>e.kind==='start').length===1&&SIM.log.find(e=>e.kind==='start').p.digest==='d1');
 check('result is not shown while running',!q('result-dialog').open);
 window.__acCloseRequested();check('active close dialog focuses keep running',q('start-close-dialog').open&&document.activeElement===q('start-close-keep'));q('start-close-dialog').querySelector('[value=cancel]').click();check('cancel close sends no close command',!SIM.log.some(e=>e.kind==='close_window'));
 FLOW.complete();document.dispatchEvent(new Event('visibilitychange'));await until(()=>q('result-dialog').open,'automatic completion result');
 check('result includes actual diff and test rows',q('result-files').textContent.includes('src/a.rs')&&q('result-tests').textContent.includes('task-1'));q('result-close').click();
 q('startup-home').click();await until(()=>!document.body.classList.contains('on-run')&&!q('start-save').disabled,'return Workspace');
 q('start-goal-list-open').click();await until(()=>document.querySelector('.goal-result'),'historical Goal row');document.querySelector('.goal-result').click();await until(()=>q('result-dialog').open,'historical result');
 check('history result keeps current Goal context',SIM.log.some(e=>e.kind==='goal_result'&&e.p.slug==='first_goal')&&SIM.home.config.active_slug==='first_goal');q('result-close').click();
 document.querySelector('.goal-folder').click();await until(()=>SIM.log.some(e=>e.kind==='goal_folder'&&e.p.open),'folder action');
 for(const d of document.querySelectorAll('dialog.slide-panel[open]'))d.close();
 q('start-new-goal').click();await until(()=>SIM.log.some(e=>e.kind==='new_goal')&&q('start-goal').value===''&&!q('start-save').disabled,'new Goal');
 check('new Goal keeps team defaults',JSON.stringify(SIM.home.config.roles)===inherited&&SIM.home.config.profiles[0].coordinator_model==='opus');
 input('start-goal','Continue a second Goal');q('start-save').click();await until(()=>!q('start-save').disabled,'second Goal saved');check('second Goal persisted before deletion '+JSON.stringify({goal:SIM.home.config.goal,saves:SIM.log.filter(e=>e.kind==='save').map(e=>e.p.config.goal)}),SIM.home.config.goal==='Continue a second Goal');
 q('start-goal-list-open').click();await until(()=>document.querySelector('.goal-trash:not(:disabled)'),'delete historical row');document.querySelector('.goal-trash:not(:disabled)').click();await until(()=>q('start-delete-goal').open,'delete confirmation');
 check('permanent deletion is explicit and cancel default',document.activeElement?.value==='cancel'&&q('start-delete-goal-path').textContent==='/tmp/sim-repo/first_goal');q('start-delete-goal-confirm').click();
 await until(()=>!q('start-delete-goal').open&&SIM.home.slugs.length===0,'pending deletion retry');
 const deleted=SIM.log.filter(e=>e.kind==='delete_goal').map(e=>e.p);check('retry preserves confirmed Goal/root and unchanged authoritative revision',deleted.length===2&&deleted.every(p=>p.slug==='first_goal'&&p.goal==='Implement the joined Goal'&&p.root==='/tmp/sim-repo/first_goal')&&deleted.every(p=>typeof p.revision==='string'&&p.revision===deleted[0].revision));
 check('inactive Goal deletion preserves new Goal',SIM.home.config.goal==='Continue a second Goal');
 for(const d of document.querySelectorAll('dialog.slide-panel[open]'))d.close();FLOW.phaseHistory();document.dispatchEvent(new Event('visibilitychange'));
 await until(()=>q('start-plan-phases').querySelector('button'),'retained phase continuation');q('start-plan-phases').querySelector('button').click();await until(()=>SIM.log.some(e=>e.kind==='next_phase')&&!q('start-save').disabled,'next phase');
 check('retained phase continuation launches no new planner',SIM.log.filter(e=>e.kind==='prepare').length===1&&SIM.home.proposal.slug_phase==='02-next');
 check('no UI runtime errors',flowErrors.length===0);return checks;
 })()`);assert(checked.length>=12);console.log(theme,width,checked.length,'joined Goal lifecycle checks passed');
}await browser.close();browser=null;}
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
