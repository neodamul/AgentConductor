// #997: real DOM on completed Goals, without real CLIs or external services.
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const{findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-completed-goal-'));let b;
 try{
  const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(findChrome(),{width:1280,height:900});
  for(const theme of ['light','dark'])for(const language of ['en','ko']){
   await b.goto(pathToFileURL(file).href);
   console.log(theme,language,await b.eval(`(async()=>{
    const q=id=>document.getElementById(id),wait=ms=>new Promise(r=>setTimeout(r,ms)),until=async f=>{for(let n=0;n<400;n++){if(f())return;await wait(20);}throw Error('completed UI did not settle');},check=(m,ok)=>{if(!ok)throw Error(m);};
    document.documentElement.dataset.theme='${theme}';AC_I18N.set('${language}');
    await until(()=>q('start-name').value==='sim');AC_I18N.set('${language}');check('requested language active',AC_I18N.current()==='${language}');
    SIM.home.run_available=true;SIM.home.run_state='completed';SIM.home.proposal=SIM.proposal('starting');SIM.home.proposal.plan=SIM.plan;
    SIM.home.slugs=[{slug:'finished',goal:SIM.home.config.goal,active:true,progress:{stage:'completed',phases:[]}}];
    SIM.room.report.state='completed';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'abc123'};
    SIM.room.report.tests=SIM.plan.tasks.map(t=>({task:t.id,exit_code:0,timed_out:false,passed:true}));
    SIM.reply.coordinator=()=>({history:[{event:'amendment',seq:1,summary:'Recorded change',payload:{}}],judgments:[{kind:'amendment',judgment:'applied',id:'old-change',seq:1,detail:'Recorded change'}],stdout:'saved verification',stderr:''});
    SIM.reply.evidence=()=>({report:SIM.room.report,test_output:[],diff:'+finished',truncated:false});
    await until(()=>q('start-save-only').disabled&&q('start-goal').readOnly);
    const startIds=['save-only','save','roles-save','prepare','regenerate','draft','read-plan','edit','run','auto-merge','coordinator','coordinator-model'];
    for(const id of startIds){const n=q('start-'+id);check('locked '+id,n.disabled||n.readOnly);check('reason '+id,!!n.title);}
    // GitHub #1040: the reusable agent registry stays manageable with a completed Goal selected.
    const cardButtons=[q('start-add'),...document.querySelectorAll('#start-profiles .profile-tools button')];
    check('#1040 agent card controls stay usable',cardButtons.length>1&&cardButtons.every(b=>b.dataset.goalReadonly!=='true'&&(!b.disabled||b.matches('.move-agent'))));
    check('#1040 the client guard lets profile saves through',!acGoalReadOnly.blocks(SIM.home,{kind:'save_profiles'}));
    check('#1040 Goal writes stay blocked',acGoalReadOnly.blocks(SIM.home,{kind:'save'})&&acGoalReadOnly.blocks(SIM.home,{kind:'optimize_goal'}));
    for(const step of ['workspace','team','review']){q('start-tab-'+step).click();check('saved stage '+step,!q('start-panel-'+step).hidden);}
    const details=q('start-tasks').querySelector('details');check('plan can expand',!!details);details.querySelector('summary').click();check('plan expanded',details.open);
    check('goal retains text',q('start-goal').value===SIM.home.config.goal);
    check('new Goal stays available',!q('start-new-goal').disabled);
    q('start-chat-button').click();await until(()=>!q('start-chat').hidden);
    SIM.reply.chat=()=>({id:'completed-question',kind:'chat'});
    q('start-chat-input').value='Explain this completed Goal';q('start-chat-form').dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));
    await until(()=>SIM.log.some(e=>e.kind==='chat'));check('chat does not mutate Goal',SIM.home.run_state==='completed');q('start-chat-close').click();
    q('start-tab-tasks').click();await until(()=>document.body.classList.contains('on-run')&&q('send-feedback').disabled);
    for(const id of ['send-feedback','reassign','set-model','schedule','resume','recover','stop','approve','reject','merge','review-revise','tool-decision-allow','tool-decision-deny']){
     const n=q(id);check('Room mutation locked '+id,n.disabled);check('Room reason '+id,!!n.title);
    }
    for(const step of ['workspace','team','review']){
     q('start-tab-'+step).click();await until(()=>!document.body.classList.contains('on-run'));check('Room -> saved '+step,!q('start-panel-'+step).hidden);
     q('start-tab-tasks').click();await until(()=>document.body.classList.contains('on-run'));
    }
    check('quota refresh available',!q('quota-refresh').disabled);
    document.querySelector('#sessions .session:not(.final-step) .session-choice').click();
    document.querySelector('[data-tab=logs]').click();check('saved logs open',!q('logs-view').hidden);
    document.querySelector('[data-tab=knowledge]').click();check('lessons open',!q('knowledge-view').hidden);
    q('open-review').click();await until(()=>!q('review-view').hidden);check('evidence was fetched',SIM.log.some(e=>e.kind==='evidence'));
    q('show-result').click();await until(()=>q('result-dialog').open);q('result-x').click();
    document.querySelector('#sessions .final-step .session-choice').click();await until(()=>document.querySelector('[data-goal-write][data-goal-readonly]'));
    check('amendment Undo locked',[...document.querySelectorAll('#activity [data-goal-write]')].every(b=>b.disabled));
    q('start-tab-workspace').click();await until(()=>!document.body.classList.contains('on-run'));
    SIM.reply.new_goal=()=>{SIM.home.run_available=false;SIM.home.run_state=null;SIM.home.proposal=null;SIM.home.slugs=[];SIM.home.config.goal='';return {changed:true};};
    q('start-new-goal').click();await until(()=>SIM.log.some(e=>e.kind==='new_goal')&&!q('start-goal').readOnly&&!q('start-save-only').disabled);
    check('new Goal is editable',!q('start-save-only').disabled);
    // GitHub #1064: the role assignments locked for the completed Goal open again for the new one.
    await until(()=>!document.querySelector('#start-panel-team tr[data-role] select')?.disabled);
    const roleAgents=[...document.querySelectorAll('#start-panel-team tr[data-role]')].map(tr=>tr.querySelector('select'));
    check('#1064 agent card tools stay usable',[...document.querySelectorAll('#start-profiles .profile-tools button')].filter(x=>!x.classList.contains('move-agent')).every(x=>!x.disabled&&!x.dataset.goalChanging));
    check('#1064 role agent choices unlock for the new Goal',roleAgents.length>0&&roleAgents.every(s=>!s.disabled&&!s.dataset.goalReadonly&&!s.dataset.goalChanging));
    const writes=SIM.log.filter(e=>e.kind==='action'&&e.p.action.kind!=='quota');check('no mutating Room dispatch',writes.length===0);
    return 'completed Goal: stage clicks, saved views, write locks, quota and New Goal passed';
   })()`));
  }
 }finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}
})().catch(e=>{console.error(e);process.exitCode=1;});
