// GitHub #828: agent cards count their sessions; the dialog stops a running session's run and cleans up
// finished ones only after a confirmation.
const fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('agent sessions skipped: no Chrome');return;}const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-sessions-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
 browser=await launch(chrome,{width:1280});await browser.goto(pathToFileURL(file).href);
 const checked=await browser.eval(`(async()=>{
  const q=id=>document.getElementById('start-agent-sessions-'+id),checks=[],check=(name,ok)=>{if(!ok)throw Error(name);checks.push(name);};
  const until=async(f,name)=>{const end=Date.now()+15000;while(!f()){if(Date.now()>end)throw Error(name);await wait(30);}};
  const row=(task,state,extra)=>Object.assign({run:'run-a',task,label:'T1 '+task,goal:'Rotate tokens',state,started_ms:Date.now()-60000,last_ms:Date.now(),note:null,cleanable:false,stoppable:false},extra);
  let data={claude:[row('t1','running',{stoppable:true}),row('t2','done',{run:'run-b',cleanable:true}),row('t3','stale',{run:'run-b',cleanable:true})]};
  SIM.reply.agent_sessions=()=>data;
  SIM.reply.stop_session=p=>{data={claude:data.claude.map(s=>s.run===p.run?{...s,state:'stopped',stoppable:false}:s)};return data;};
  SIM.reply.clean_sessions=p=>{const gone=p.sessions.map(x=>x.join('/'));data={claude:data.claude.filter(s=>!gone.includes(s.run+'/'+s.task))};return data;};
  await until(()=>document.querySelector('[data-sessions="claude"]'),'chip');await acSessions.refresh();
  await until(()=>document.querySelector('[data-sessions="claude"] .session-count')?.textContent==='3','chip count');
  const chip=document.querySelector('[data-sessions="claude"]');
  check('the agent card counts its sessions and shows its work states (#869)',chip.textContent.startsWith(acMessage('sessions.chip'))&&[...chip.parentElement.querySelectorAll('[data-state-tag]')].map(t=>t.dataset.stateTag+t.querySelector('.state-count').textContent).join()==='running1,failed1,done1'&&chip.previousElementSibling.dataset.stateTag==='done');
  chip.click();await until(()=>document.getElementById('start-agent-sessions').open&&q('list').children.length===3,'dialog');
  const rows=()=>[...q('list').querySelectorAll('.session-row')];
  check('rows show state, task and Goal',rows()[0].querySelector('.session-state.running')&&rows()[0].textContent.includes('Rotate tokens'));
  check('running sessions stop, finished ones clean up',rows()[0].querySelector('button').textContent===acMessage('sessions.stop')&&rows()[1].querySelector('button').textContent===acMessage('sessions.clean')&&q('clean-finished').textContent===acMessage('sessions.clean_finished_count',{value1:2}));
  rows()[0].querySelector('button').click();await until(()=>rows()[0].querySelector('.session-state.stopped'),'stopped');
  check('stop sends the run and keeps the session as stopped',SIM.log.some(e=>e.kind==='stop_session'&&e.p.run==='run-a'));
  q('clean-finished').click();check('clean-up asks first',!q('confirm').hidden&&!SIM.log.some(e=>e.kind==='clean_sessions'));
  q('confirm-cancel').click();check('cancel keeps every session',q('confirm').hidden&&rows().length===3);
  rows()[1].querySelector('button').click();q('confirm-ok').click();await until(()=>rows().length===2,'cleaned');
  check('confirmed clean-up removes that session and updates the card',SIM.log.some(e=>e.kind==='clean_sessions'&&e.p.sessions.length===1)&&chip.querySelector('.session-count').textContent==='2');
  // GitHub #962: a double click cleans once, focus stays in the dialog, and the confirm button is the danger button.
  {const before=SIM.log.filter(e=>e.kind==='clean_sessions').length;SIM.hang.clean_sessions=true;q('clean-finished').click();const danger=getComputedStyle(q('confirm-ok')).backgroundColor!==getComputedStyle(q('confirm-cancel')).backgroundColor;q('confirm-ok').click();q('confirm-ok').click();await until(()=>SIM.waiters.clean_sessions?.length,'clean sent');const locked=q('confirm-ok').disabled&&q('clean-finished').disabled;delete SIM.hang.clean_sessions;SIM.waiters.clean_sessions.splice(0).forEach(go=>go());await until(()=>rows().length===1,'cleaned once');check('#962: confirming twice sends one clean-up, locks the dialog while it runs, keeps focus in the dialog, and shows a danger button',SIM.log.filter(e=>e.kind==='clean_sessions').length===before+1&&locked&&document.getElementById('start-agent-sessions').contains(document.activeElement)&&danger);}
  data={claude:[row('t9','done',{cleanable:false})]};await acSessions.refresh();
  check('a finished session in an active run cannot be cleaned yet',rows()[0].querySelector('button').disabled&&q('clean-finished').disabled);
  q('close').click();data={};await acSessions.refresh();chip.click();
  check('an agent without sessions shows the empty state',q('list').textContent===acMessage('sessions.empty'));q('close').click();
  return checks;
 })()`);
 console.log(checked.length,'agent sessions checks passed');
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
