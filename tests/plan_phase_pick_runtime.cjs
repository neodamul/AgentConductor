// GitHub #736: a phased Goal's plan step lists the tasks of ONE Phase (the one picked in the rail), every row has its
// expand button and state, and the rail opens only the picked Phase.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('plan phase pick skipped: no Chrome');process.exit(77);}
const SIM_TASK_COUNT=3;
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-plan-pick-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
const preset=`SIM.home.proposal=SIM.proposal('ready');SIM.home.slugs=[{slug:'g',goal:'g',active:true,plan:{phases:[
 {id:'p1',brief:'Tasks:\\n- a1\\n- a2',specs:[{id:'a1',agent:'claude',prompt:'Write the spec',paths:['docs/spec.md'],depends_on:[],test_command:['true']},{id:'a2',agent:'claude',prompt:'Review the spec',paths:['docs/review.md'],depends_on:['a1'],test_command:[]}]},
 {id:'p2',brief:'Tasks:\\n- b1',specs:[{id:'b1',agent:'claude',prompt:'Build it',paths:['src/x'],depends_on:[],test_command:['true']}]}]},
 progress:{stage:'verification',current_phase:'p2',phases:[{id:'p1',name:'Spec',stage:'completed',task:['a1','a2'],completed:['a1','a2'],verification:['a1','a2']},{id:'p2',name:'Build',stage:'plan',task:['b1'],completed:[],verification:[]}]}}];SIM.home.proposal.slug='g';SIM.home.proposal.slug_phase='p2';`;
(async()=>{try{const b=await launch(chrome,{width:Number(process.env.PARITY_WIDTH)||1280,height:900});try{
 const url=pathToFileURL(path.join(dir,'app.html')).href;await b.goto(url);await b.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(preset)})`);await b.goto(url);await wait(1500);
 await b.eval("document.getElementById('start-tab-review').click()");await wait(800);
 await b.eval("document.documentElement.dataset.theme="+JSON.stringify(process.env.PARITY_THEME||'light'));
 const first=await b.eval("({rows:[...document.querySelectorAll('#start-plan-phases .plan-phase')].length,tasks:[...document.querySelectorAll('#start-tasks details.startup-task')].filter(d=>d.getBoundingClientRect().height>0).length,heading:document.querySelector('#start-panel-review .eyebrow')?.textContent,briefs:document.querySelectorAll('#start-plan-phases .plan-phase-brief').length,note:document.getElementById('start-plan-phases').textContent.includes(acMessage('startup.these_tasks_are_open_for_review_and_editing')),railOpen:[...document.querySelectorAll('#rail-outline-body .phase-tree')].map(t=>t.open)})");
 if(first.rows!==0||first.tasks!==SIM_TASK_COUNT||first.briefs||first.note||!/Phase 2\/2.*Build/.test(first.heading))fail('the plan step must list only the current Phase: '+JSON.stringify(first));
 if(JSON.stringify(first.railOpen)!=='[false,true]')fail('the rail must open only the current Phase: '+JSON.stringify(first.railOpen));
 await b.eval("document.querySelector('#start-tasks textarea[data-key=prompt]').value='Preserved operator edit'");
 await b.eval("document.querySelectorAll('#rail-outline-body .phase-tree summary')[0].click()");await wait(500);
 const second=await b.eval("({tasks:[...document.querySelectorAll('#start-plan-phases .plan-phase-tasks li')].map(l=>l.textContent.replace(/\\s+/g,' ')),heading:document.querySelector('#start-plan-phases .plan-phase summary strong')?.textContent,railOpen:[...document.querySelectorAll('#rail-outline-body .phase-tree')].map(t=>t.open),toggles:document.querySelectorAll('#start-plan-phases .task-toggle').length})");
 if(!/Spec/.test(second.heading)||second.tasks.length!==2)fail('picking Phase 1 in the rail must list its two tasks: '+JSON.stringify(second));
 if(JSON.stringify(second.railOpen)!=='[true,false]')fail('the rail must open only the picked Phase: '+JSON.stringify(second.railOpen));
 if(await b.eval("!!document.querySelector('#start-plan-phases .plan-phase-brief')"))fail('history must omit task-ID-only briefs');
 if(second.toggles!==2||!second.tasks.every(t=>/펼치기|Expand/i.test(t)))fail('every task row needs its expand button: '+JSON.stringify(second));
 if(!second.tasks.every(t=>/완료|Completed/i.test(t)))fail('every task row shows its state: '+JSON.stringify(second.tasks));
 await b.eval("document.querySelector('#start-plan-phases .plan-phase-tasks li').click()");
 const open=await b.eval("(()=>{const d=document.querySelector('#start-plan-phases .plan-phase-tasks li .task-detail');return !d.hidden&&/Write the spec/.test(d.textContent)&&/docs\\/spec\\.md/.test(d.textContent);})()");
 if(!open)fail('expanding a row shows the task detail');
 if(await b.eval("[...document.querySelectorAll('#start-tasks details.startup-task')].some(d=>d.getBoundingClientRect().height>0)"))fail('the editable plan must stay hidden while another Phase is picked');
 await b.eval("document.querySelectorAll('#rail-outline-body .phase-tree summary')[1].click()");await wait(300);
 if(!await b.eval("document.querySelector('#start-tasks textarea[data-key=prompt]').value==='Preserved operator edit'&&!document.getElementById('start-tasks').hidden&&!document.querySelector('#start-plan-phases .plan-phase')"))fail('returning to review preserves editable task input without a duplicate wrapper');
 await b.eval("AC_I18N.set('en')");
 if(!await b.eval("/Step 3.*Plan.*Phase 2\\/2.*Build/.test(document.getElementById('start-plan-eyebrow').textContent)"))fail('phase heading follows the selected UI language');
 if(!await b.eval("!acPlanReview.meaningfulBrief({brief:'Tasks:\\n- a1\\n- a2',specs:[{id:'a1'},{id:'a2'}]})&&acPlanReview.meaningfulBrief({brief:'Keep the authentication contract.\\nTasks:\\n- a1',specs:[{id:'a1'}]})"))fail('hide task-ID-only briefs but keep meaningful history context');
 await b.eval("SIM.home.slugs[0].plan.phases[0].brief='Preserve authentication compatibility'");await wait(5500);
 await b.eval("document.querySelectorAll('#rail-outline-body .phase-tree summary')[0].click()");await wait(300);
 if(!await b.eval("(()=>{const brief=document.querySelector('#start-plan-phases .plan-phase-brief');return brief?.textContent==='Preserve authentication compatibility'&&brief.getBoundingClientRect().height>0;})()"))fail('meaningful history brief remains visible above its tasks');
 await b.eval("SIM.home.slugs[0].progress.current_phase=null;SIM.home.slugs[0].progress.stage='completed';SIM.home.slugs[0].progress.phases.forEach(p=>{p.stage='completed';p.completed=[...p.task]});SIM.home.proposal.slug_phase='p2'");await wait(5500);
 const completed=await b.eval("({heading:document.querySelector('#start-plan-phases .plan-phase summary strong')?.textContent,railOpen:[...document.querySelectorAll('#rail-outline-body .phase-tree')].map(t=>t.open),status:document.querySelector('#rail-outline-body .outline-run-status')?.textContent})");
 if(!/Build/.test(completed.heading)||JSON.stringify(completed.railOpen)!=='[false,true]'||!/완료|Done|Completed/i.test(completed.status))fail('a completed Goal opens its final Phase and completed status: '+JSON.stringify(completed));
 console.log('plan phase pick: one Phase listed, rail follows, rows expand and show their state');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
