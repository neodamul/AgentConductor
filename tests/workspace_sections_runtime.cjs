// #427: real assembled Workspace section ownership, preserved actions and full briefing.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {launch,findChrome}=require('./design_parity/cdp.cjs'),chrome=findChrome();if(!chrome){console.log('workspace sections skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-workspace-sections-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const preset=`SIM.home.config.goal='Goal A';SIM.home.slugs=[{slug:'goal-a',goal:'Goal A',active:true,root:'/tmp/goal-a',progress:{stage:'planned',phases:[]}}];SIM.reply.workspace_briefing=()=>({state:'running',observed_at_ms:Date.now(),sessions:[{agent:'claude',task:'task-1',state:'running',work:'Implement authentication',session_id:'session-1',model:'opus',attempt:2,process:{state:'present',observed_at_ms:Date.now()},last_activity:{kind:'progress',freshness:'fresh',text:'Writing tests'},memory_report:{summary:'Preserve rotation state',source:'local',freshness:'fresh'},attention:'Review edge cases',next_action:'Wait for results'}],approvals:[{kind:'tool',task:'task-1',status:'pending'}]});window.sectionFixture=true;`;
(async()=>{try{for(const width of [840,1280]){const b=await launch(chrome,{width,height:960});try{for(const theme of ['light','dark']){
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);await b.eval(`PRESET(${JSON.stringify(preset)})`);
 for(let i=0;i<50;i++){let ready=false;try{ready=await b.eval('!!window.sectionFixture && !!document.querySelector("#start-profiles .startup-profile")');}catch{}if(ready)break;await new Promise(r=>setTimeout(r,50));}
 const checks=await b.eval(`(async()=>{
  const out=[],check=(n,v)=>{if(!v)throw Error(n);out.push(n);},q=id=>document.getElementById(id),wait=()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
  document.documentElement.dataset.theme='${theme}';
  const workspace=q('start-workspace-section'),goal=q('start-goal-section');check('distinct Workspace and Goal sections',!!workspace&&!!goal);check('Workspace title reflects the current step '+JSON.stringify({actual:q('start-page-title')?.textContent,expected:acMessage('startup.workspace_settings'),notice:q('start-notice')?.textContent}),q('start-page-title')?.textContent===acMessage('startup.workspace_settings'));
  const info=q('start-goal-info'),tabs=q('start-step-tabs');check('fixed Goal information row',!!info&&info.getBoundingClientRect().height===36);
  const tabTop=tabs.getBoundingClientRect().top;info.firstElementChild.textContent='Progress detail '.repeat(40);await wait();check('information updates do not move step tabs',tabs.getBoundingClientRect().top===tabTop);info.firstElementChild.textContent='';
  check('Workspace owns common fields',['start-name','start-coordinator','start-repository','start-worktrees'].every(id=>workspace.contains(q(id))));
  check('Goal owns objective, New Goal and imports',['start-goal','start-new-goal','start-mcp-controls','start-optimize-button'].every(id=>goal.contains(q(id))));
  check('page heading holds the Goal list and briefing but not New Goal (#824)',!document.querySelector('.startup-heading #start-new-goal')&&!!document.querySelector('.startup-heading #start-goal-list-open')&&!!document.querySelector('.startup-heading #brief-team'));
  check('current Goal is actual selected state',q('start-current-goal').textContent==='Goal A'&&!!q('start-current-goal-state').textContent);
  check('briefing disabled without a run',q('brief-team').disabled);
  q('start-goal-list-open').click();check('existing Goal list opens',q('start-goals').open);q('start-goals-close').click();
  q('start-tab-team').click();check('Goal list stays reachable on Team',q('start-goal-list-open').getBoundingClientRect().width>0);check('Team title is the actual Goal',q('start-page-title').textContent==='Goal A');
  q('start-tab-workspace').click();check('Goal list stays in the heading on Workspace (#824)',document.querySelector('.startup-heading').contains(q('start-goal-list-open')));
  SIM.home.run_available=true;SIM.home.run_state='running';const until=Date.now()+12000;while(q('brief-team').disabled&&Date.now()<until)await new Promise(r=>setTimeout(r,30));
  check('existing run enables briefing',!q('brief-team').disabled);q('brief-team').click();
  const briefingUntil=Date.now()+5000;while(!q('briefing-dialog').open&&Date.now()<briefingUntil)await new Promise(r=>setTimeout(r,30));
  check('Workspace opens full briefing '+JSON.stringify({open:q('briefing-dialog').open,notice:q('start-notice').textContent,brief:q('briefing-dialog').textContent,requests:SIM.log.slice(-5).map(x=>x.kind)}),q('briefing-dialog').open&&['Implement authentication','Writing tests','Preserve rotation state','Review edge cases','Wait for results'].every(t=>q('briefing-dialog').textContent.includes(t)));
  check('briefing does not open or start a run',!SIM.log.some(x=>x.kind==='open_run'||x.kind==='start'));
  q('briefing-dialog').close();await wait();q('start-tab-tasks').click();
  const end=Date.now()+5000;while(!document.body.classList.contains('on-run')&&Date.now()<end)await new Promise(r=>setTimeout(r,30));
  check('Room retains same Goal list button',!!document.querySelector('#run-screen .goal-bar #start-goal-list-open'));
  check('single briefing and Goal controls',document.querySelectorAll('#brief-team').length===1&&document.querySelectorAll('#start-goal-list-open').length===1);
  q('startup-home').click();const homeUntil=Date.now()+5000;while(document.body.classList.contains('on-run')&&Date.now()<homeUntil)await new Promise(r=>setTimeout(r,30));
  check('return from Room restores both Goal controls to the heading',document.querySelector('.startup-heading').contains(q('start-goal-list-open'))&&document.querySelector('.startup-heading').contains(q('brief-team')));q('brief-team').click();const again=Date.now()+5000;while(!q('briefing-dialog').open&&Date.now()<again)await new Promise(r=>setTimeout(r,30));check('Workspace briefing still works after Room',q('briefing-dialog').open);q('briefing-dialog').close();
  return out;
 })()`);console.log(theme,width,checks.length,'Workspace section checks passed');
}}finally{await b.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
