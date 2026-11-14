// #420 / #989: joined UI journey with operator intervention, exact approval/merge and restored Goal state.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs'),preset=require('./ui_sim/lifecycle.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('Goal lifecycle skipped: no Chrome');return;}const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-goal-lifecycle-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
for(const width of [840,960,1280]){browser=await launch(chrome,{width});for(const theme of ['light','dark']){
 await browser.goto(pathToFileURL(file).href);await browser.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(preset)})`);await browser.goto(pathToFileURL(file).href);
 const checked=await browser.eval(`(async()=>{
 const q=id=>document.getElementById(id),checks=[],check=(name,ok)=>{if(!ok)throw Error(name);checks.push(name);};
 const until=async(f,name)=>{const end=Date.now()+15000;while(!f()){if(Date.now()>end)throw Error(name+' '+JSON.stringify({logs:SIM.log.slice(-10).map(x=>x.kind),notice:q('start-notice').textContent,errors:flowErrors,open:[...document.querySelectorAll('dialog[open]')].map(d=>d.id)}));await wait(30);}};
 const input=(id,value)=>{q(id).value=value;q(id).dispatchEvent(new Event('input'));};
 document.documentElement.dataset.theme='${theme}';await until(()=>q('start-new-workspace').open,'first Workspace dialog');
 input('start-new-name','Lifecycle Workspace');q('start-new-create').click();await until(()=>!q('start-new-workspace').open&&!q('start-save').disabled&&q('start-name').value==='Lifecycle Workspace','created Workspace settles');
 check('create Workspace uses real UI request',SIM.log.some(e=>e.kind==='create_workspace'&&e.p.name==='Lifecycle Workspace'));
 const originalPrepare=SIM.reply.prepare;
 SIM.reply.prepare=p=>{SIM.home.job={id:'journey-plan',kind:'plan',state:'running',started_ms:Date.now()};setTimeout(()=>{originalPrepare(p);SIM.home.job=null;document.dispatchEvent(new Event('visibilitychange'));},900);return {started:true};};
 SIM.reply.action=p=>{const a=p.action;SIM.room.job={id:a.kind+Date.now(),kind:a.kind,state:"completed"};SIM.room.revision='journey-'+Date.now();if(a.kind==='stop'){SIM.room.report.state='stopped';SIM.home.run_state='stopped';SIM.room.report.tasks.filter(t=>t.status==='running').forEach(t=>t.status='stopped');}if(a.kind==='resume'){SIM.room.report.state='running';SIM.home.run_state='running';SIM.room.report.tasks.filter(t=>t.status==='stopped').forEach(t=>t.status='running');}if(a.kind==='reassign'){const t=SIM.room.report.plan.tasks.find(t=>t.id===a.task);if(t){t.agent=a.agent;t.model=a.model;}}if(a.kind==='tool_reject')SIM.room.tool_requests=[];if(a.kind==='approve')SIM.room.report.state='approved';if(a.kind==='merge')FLOW.complete();return {accepted:true};};

 input('start-goal','Implement the joined Goal');let inherited=JSON.stringify(SIM.home.config.roles);q('start-save').click();await until(()=>!q('start-panel-team').hidden&&!q('start-prepare').disabled,'saved Goal and Team');

 const field=name=>document.querySelector('tr[data-role="development"] [data-role-field="'+name+'"]');
 field('model').value='gpt';field('model').dispatchEvent(new Event('change',{bubbles:true}));
 acChoices.open(field('fallback'));await until(()=>document.querySelector("#ac-choice-menu button"),"role choices open");document.querySelector('#ac-choice-menu [data-value="claude"]').click();
 acChoices.open(field('fallback-model'));await until(()=>document.querySelector("#ac-choice-menu button"),"role choices open");document.querySelector('#ac-choice-menu [data-value="opus"]').click();
 q('start-roles-save').click();await until(()=>!q('start-roles-save').disabled&&SIM.home.config.roles.development[0].fallback?.model==='opus','role fallback saved');
 inherited=JSON.stringify(SIM.home.config.roles);check('role fallback and model saved before planning',SIM.home.config.roles.development[0].fallback.agent==='claude');
 q('start-prepare').click();check('planning does not open a confirmation',!q('start-planning-confirm')?.open);
 await until(()=>SIM.home.job?.kind==='plan','planner is pending');check('execution unavailable during planning',q('start-run').disabled);

 await until(()=>SIM.home.proposal?.phase==='ready'&&!q('start-run').disabled,'review ready');check('planning uses registered coordinator and model',SIM.home.config.coordinator==='claude'&&SIM.home.config.profiles[0].coordinator_model==='opus');
 check('the plan screen has no legacy approval evidence disclosure',!q('start-approval-evidence')&&!q('start-consent-evidence'));
 check('Remove task explains manual additions',document.querySelectorAll('.task-remove').length>0&&[...document.querySelectorAll('.task-remove')].every(b=>b.title===acMessage('startup.remove_task_hint')));
 q('start-run').click();check('run does not open a confirmation',!q('start-consent')?.open);
 await until(()=>document.body.classList.contains('on-run')&&document.querySelectorAll('.session-choice .task-number').length===3,'running Room');
 check('run starts only after exact consent',SIM.log.filter(e=>e.kind==='start').length===1&&SIM.log.find(e=>e.kind==='start').p.digest==='d1');
 check('result is not shown while running',!q('result-dialog').open);
 q('stop').click();await until(()=>q('decision-dialog').open,'stop confirmation');q('dialog-reason').value='Pause for assignment review';q('dialog-confirm').click();
 await until(()=>SIM.room.report.state==='stopped'&&!q('resume').disabled,'stopped run settles');
 const agent=q('agent-select'),model=q('model-select');agent.value='claude';agent.dispatchEvent(new Event('change',{bubbles:true}));
 await until(()=>[...model.options].some(o=>o.value==='opus'),'alternate model available');model.value='opus';q('reassign').click();
 await until(()=>SIM.log.some(e=>e.kind==='action'&&e.p.action.kind==='reassign'),'reassignment dispatched');
 const assigned=SIM.log.find(e=>e.kind==='action'&&e.p.action.kind==='reassign').p.action;
 check('stopped task reassigns alternate model together',assigned.agent==='claude'&&assigned.model==='opus');
 await until(()=>!q('resume').disabled,'resume ready');q('resume').click();await until(()=>SIM.room.report.state==='running','run resumed');
 SIM.room.tool_requests=[{id:'journey-tool',status:'pending',created_at:Math.floor(Date.now()/1000),context:{task:'task-1'},request:{tool:'Bash',input:{command:'fixture-network-command'}}}];
 document.dispatchEvent(new Event('visibilitychange'));await until(()=>!q('tool-decision').hidden,'tool decision requested');q('tool-decision-deny').click();
 await until(()=>SIM.log.some(e=>e.kind==='action'&&e.p.action.kind==='tool_reject'),'tool denied');
 check('tool denial binds the exact request',SIM.log.find(e=>e.kind==='action'&&e.p.action.kind==='tool_reject').p.action.sha==='journey-tool');

 window.__acCloseRequested();check('active close dialog focuses keep running',q('start-close-dialog').open&&document.activeElement===q('start-close-keep'));q('start-close-dialog').querySelector('[value=cancel]').click();check('cancel close sends no close command',!SIM.log.some(e=>e.kind==='close_window'));

 SIM.room.report.state='waiting_approval';SIM.home.run_state='waiting_approval';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'a'.repeat(40)};SIM.room.report.diff_hash='d'.repeat(64);SIM.room.report.tests=[{task:'task-1',exit_code:0,timed_out:false,output:'passed'}];SIM.room.revision='journey-review';
 document.dispatchEvent(new Event('visibilitychange'));await until(()=>!q('review-footer').hidden&&!q('review-primary').disabled,'review ready');
 if(q('review-primary').dataset.mode==='open'){q('review-primary').click();}
 await until(()=>q('review-primary').dataset.mode==='approve'&&!q('review-primary').disabled,'evidence approval ready');q('review-primary').click();
 await until(()=>q('decision-dialog').open,'candidate approval confirmation');check('approval displays exact candidate',q('dialog-evidence').textContent.includes('a'.repeat(40)));q('dialog-confirm').click();
 await until(()=>q('review-primary').dataset.mode==='merge'&&!q('review-primary').disabled,'approved merge ready');q('review-primary').click();
 await until(()=>q('decision-dialog').open,'merge confirmation');q('dialog-confirm').click();await until(()=>q('result-dialog').open,'approved merge completion result');
 check('human approval and merge dispatched once each',SIM.log.filter(e=>e.kind==='action'&&e.p.action.kind==='approve').length===1&&SIM.log.filter(e=>e.kind==='action'&&e.p.action.kind==='merge').length===1);

 check('result includes actual diff and test rows',q('result-files').textContent.includes('src/a.rs')&&q('result-tests').textContent.includes('task-1'));q('result-close').click();
 q('startup-home').click();await until(()=>!document.body.classList.contains('on-run')&&!q('start-save').disabled,'return Workspace');
 q('start-goal-list-open').click();await until(()=>document.querySelector('.goal-result'),'historical Goal row');document.querySelector('.goal-result').click();await until(()=>q('result-dialog').open,'historical result');
 check('history result keeps current Goal context',SIM.log.some(e=>e.kind==='goal_result'&&e.p.slug==='first_goal')&&SIM.home.config.active_slug==='first_goal');q('result-close').click();
 document.querySelector('.goal-folder').click();await until(()=>SIM.log.some(e=>e.kind==='goal_folder'&&e.p.open),'folder action');
 for(const d of document.querySelectorAll('dialog.slide-panel[open]'))d.close();
 q('start-new-goal').click();await until(()=>SIM.log.some(e=>e.kind==='new_goal')&&q('start-goal').value===''&&!q('start-save').disabled,'new Goal');
 check('a new Goal clears the previous plan task-count introduction',q('start-plan-note').textContent===acMessage('startup.generate_a_plan_then_review_each_role_scope'));
 check('new Goal keeps team defaults',JSON.stringify(SIM.home.config.roles)===inherited&&SIM.home.config.profiles[0].coordinator_model==='opus');
 input('start-goal','Continue a second Goal');q('start-save').click();await until(()=>!q('start-save').disabled,'second Goal saved');check('second Goal persisted before deletion '+JSON.stringify({goal:SIM.home.config.goal,saves:SIM.log.filter(e=>e.kind==='save').map(e=>e.p.config.goal)}),SIM.home.config.goal==='Continue a second Goal');
 q('start-goal-list-open').click();await until(()=>document.querySelector('.goal-trash:not(:disabled)'),'delete historical row');document.querySelector('.goal-trash:not(:disabled)').click();await until(()=>q('start-delete-goal').open,'delete confirmation');
 check('permanent deletion is explicit and cancel default',document.activeElement?.value==='cancel'&&q('start-delete-goal-path').textContent==='/tmp/sim-repo/first_goal');q('start-delete-goal-confirm').click();
 await until(()=>!q('start-delete-goal').open&&SIM.home.slugs.length===0,'pending deletion retry');
 const deleted=SIM.log.filter(e=>e.kind==='delete_goal').map(e=>e.p);check('retry preserves confirmed Goal/root and unchanged authoritative revision',deleted.length===2&&deleted.every(p=>p.slug==='first_goal'&&p.goal==='Implement the joined Goal'&&p.root==='/tmp/sim-repo/first_goal')&&deleted.every(p=>typeof p.revision==='string'&&p.revision===deleted[0].revision));
 check('inactive Goal deletion preserves new Goal',SIM.home.config.goal==='Continue a second Goal');
 for(const d of document.querySelectorAll('dialog.slide-panel[open]'))d.close();FLOW.phaseHistory();document.dispatchEvent(new Event('visibilitychange'));
 await until(()=>q('start-plan-phases').querySelector('button'),'retained phase continuation');q('start-plan-phases').querySelector('button').click();await until(()=>SIM.log.some(e=>e.kind==='next_phase')&&!q('start-save').disabled,'next phase');
 q('start-tab-review').click();await until(()=>!q('start-panel-review').hidden&&q('start-goal-info').querySelector('.goal-info-content').textContent.includes('Phase'),'multi-Phase plan step');
 check('plan heading uses the Goal-wide Phase position '+q('start-goal-info').textContent,q('start-goal-info').textContent.includes('Phase 2/2'));
 check('selected task summary uses the same Phase position',q('start-plan-summary').querySelector('.plan-position').textContent.includes('Phase 2/2'));
 check('retained phase continuation launches no new planner',SIM.log.filter(e=>e.kind==='prepare').length===1&&SIM.home.proposal.slug_phase==='02-next');
 // GitHub #812: the Phase completion popup continues into the next Phase's plan; without one it offers the Goal list.
 {const next=await window.acNextPhase.info();check('the popup learns the next Phase',next&&next.id==='02-next'&&next.number===2&&next.total===2);
  const data={goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'',tests:[],details:[]};acResultView.show({...data,nextPhase:next});
  check('a Phase with Phases left offers the next Phase plan',document.getElementById('result-close-label').textContent===acMessage('room.result_next_phase',{value1:2,value2:2})&&document.getElementById('result-eyebrow').textContent===acMessage('room.result_eyebrow_phase',{value1:1,value2:2}));
  const before=SIM.log.filter(e=>e.kind==='next_phase').length;document.getElementById('result-close').click();await until(()=>!document.getElementById('result-dialog').open&&!q('start-panel-review').hidden,'next Phase plan step');
  check('the next Phase plan opens in step 3 without a second planner',!q('start-panel-review').hidden&&SIM.log.filter(e=>e.kind==='next_phase').length===before&&SIM.log.filter(e=>e.kind==='prepare').length===1);
  acResultView.show(data);check('the last Phase offers the Goal list',document.getElementById('result-close-label').textContent===acMessage('room.result_to_goals')&&document.getElementById('result-eyebrow').textContent===acMessage('room.result_eyebrow'));document.getElementById('result-dialog').close();}
 check('no UI runtime errors',flowErrors.length===0);return checks;
 })()`);assert(checked.length>=12);console.log(theme,width,checked.length,'continuous intervention journey checks passed');
const saved=await browser.eval('({home:SIM.home,room:SIM.room})');
const restored=preset+';Object.assign(SIM.home,'+JSON.stringify(saved.home)+');Object.assign(SIM.room,'+JSON.stringify(saved.room)+');';
await browser.eval('sessionStorage.setItem("SIM_PRESET",'+JSON.stringify(restored)+')');
await browser.goto(pathToFileURL(file).href);
const reopened=await browser.eval(`(async()=>{const q=id=>document.getElementById(id);for(let i=0;i<300&&q('start-name').value!=='Lifecycle Workspace';i++)await wait(20);const checks={workspace:q('start-name').value==='Lifecycle Workspace',goal:q('start-goal').value==='Continue a second Goal',fallback:SIM.home.config.roles.development[0].fallback?.model==='opus',history:!SIM.home.slugs.some(g=>g.slug==='first_goal'),noSideEffects:!SIM.log.some(e=>['create_workspace','prepare','start','new_goal','delete_goal'].includes(e.kind)),errors:flowErrors.length===0};if(!Object.values(checks).every(Boolean))throw Error('Restored journey '+JSON.stringify(checks));return checks;})()`);
console.log(theme,width,Object.keys(reopened).length,'restored journey checks passed');
}await browser.close();browser=null;}
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
