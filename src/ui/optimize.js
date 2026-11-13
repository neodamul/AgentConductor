// DD-188 (GitHub #180): rewrite the goal with the bundled prompt-optimizer skill; the operator reviews the proposal before it replaces the goal text.
function optimizeUi({q,request,state,notice,busy,applyGoal,save,capture,draft,showStep,stale=()=>''}) {
 let waiting=null,shown=null;
 // GitHub #273 (DD-208): a rewrite in progress shows its elapsed time and is followed again after the page reopens.
 const refresh=()=>{q('optimize-again').disabled=!!waiting||busy()||!q('optimize-answers').value.trim();q('optimize-answers').disabled=!!waiting;q('optimize-use').disabled=!!waiting;q('optimize-button').disabled=!!waiting||busy()||!q('goal').value.trim();if(typeof AC_PRESS!=='undefined')for(const id of ['optimize-button','optimize-again'])AC_PRESS.hold(q(id),!!waiting);if(waiting){const late=stale();q('optimize-status').textContent=acMessage('startup.optimize_working')+' '+AC_ELAPSED(waiting.at)+(late?' · '+late:'');q('optimize-dialog-status').textContent=q('optimize-status').textContent;}};
 function fill(list,title,items){title.hidden=!items.length;list.hidden=!items.length;list.replaceChildren(...items.map(text=>{const li=document.createElement('li');li.textContent=text;return li;}));}
 function show(proposal){shown=proposal;q('optimize-goal').textContent=proposal.optimized;fill(q('optimize-changes'),q('optimize-changes-title'),proposal.changes||[]);fill(q('optimize-questions'),q('optimize-questions-title'),proposal.questions||[]);q('optimize-answers-section').hidden=!(proposal.questions||[]).length;q('optimize-answers').value='';refresh();if(!q('optimize').open)q('optimize').showModal();}
 function done(text){waiting=null;q('optimize-status').textContent=text||'';q('optimize-dialog-status').textContent=text||'';refresh();}
 // GitHub #680: the coordinator that rewrites the goal is the one saved for the workspace, so the unsaved draft is saved first (as the document preview does).
 async function start(again=false){if(waiting||busy()||!q('goal').value.trim()||again&&(!shown||!q('optimize-answers').value.trim()))return;const input={goal:again?shown.optimized:q('goal').value,...(again?{answers:JSON.stringify({questions:shown.questions,answers:q('optimize-answers').value.trim()})}:{})};capture();const c=draft();if(!c.coordinator||!c.profiles?.some(p=>p.id===c.coordinator)){notice(()=>acMessage('startup.optimize_coordinator_required'),true);showStep?.('workspace',true);q('coordinator')?.focus?.();return;}waiting={at:Date.now()};refresh();try{if(!await save()){done('');return;}const r=await request({kind:'optimize_goal',revision:state().revision,...input});waiting={job:r.id,at:waiting.at};}catch(e){done(e.message);notice(e.message,true);}}
 function update(){const s=state();if(!waiting&&s?.job?.kind==='optimize'&&s.job.state==='running')waiting={job:s.job.id,at:s.job.started_ms||Date.now()};refresh();if(!waiting?.job||!s?.job||s.job.id!==waiting.job)return;const job=s.job;
  if(job.state==='completed'&&s.optimize?.job===job.id){done('');show(s.optimize.proposal);}
  else if(['failed','interrupted','stopped'].includes(job.state)){done(acMessage('startup.optimize_error',{value1:job.error||job.state}));notice(acMessage('startup.optimize_error',{value1:job.error||job.state}),true);}}
 q('optimize-button').onclick=()=>start();
 q('optimize-again').onclick=()=>start(true);q('optimize-answers').oninput=refresh;
 q('optimize-use').onclick=()=>{if(shown)applyGoal(shown.optimized);shown=null;q('optimize').close();};
 q('optimize-keep').onclick=()=>{shown=null;q('optimize').close();};
 return {update,proposal:()=>shown};
}
