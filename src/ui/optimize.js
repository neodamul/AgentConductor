// DD-188 (GitHub #180): rewrite the goal with the bundled prompt-optimizer skill; the operator reviews the proposal before it replaces the goal text.
function optimizeUi({q,request,state,notice,busy,applyGoal}) {
 let waiting=null,shown=null;
 const refresh=()=>{q('optimize-button').disabled=!!waiting||busy()||!q('goal').value.trim();};
 function fill(list,title,items){title.hidden=!items.length;list.hidden=!items.length;list.replaceChildren(...items.map(text=>{const li=document.createElement('li');li.textContent=text;return li;}));}
 function show(proposal){shown=proposal;q('optimize-goal').textContent=proposal.optimized;fill(q('optimize-changes'),q('optimize-changes-title'),proposal.changes||[]);fill(q('optimize-questions'),q('optimize-questions-title'),proposal.questions||[]);q('optimize').showModal();}
 function done(text){waiting=null;q('optimize-status').textContent=text||'';refresh();}
 async function start(){if(waiting||!q('goal').value.trim())return;waiting={};AC_I18N.text(q('optimize-status'),()=>acMessage('startup.optimize_working'));refresh();try{const r=await request({kind:'optimize_goal',revision:state().revision,goal:q('goal').value});waiting={job:r.id};}catch(e){done('');notice(e.message,true);}}
 function update(){refresh();const s=state();if(!waiting?.job||!s?.job||s.job.id!==waiting.job)return;const job=s.job;
  if(job.state==='completed'&&s.optimize?.job===job.id){done('');show(s.optimize.proposal);}
  else if(['failed','interrupted','stopped'].includes(job.state)){done('');notice(acMessage('startup.optimize_error',{value1:job.error||job.state}),true);}}
 q('optimize-button').onclick=start;
 q('optimize-use').onclick=()=>{if(shown)applyGoal(shown.optimized);shown=null;q('optimize').close();};
 q('optimize-keep').onclick=()=>{shown=null;q('optimize').close();};
 return {update,proposal:()=>shown};
}
