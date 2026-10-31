// DD-188 (GitHub #180): rewrite the goal with the bundled prompt-optimizer skill; the operator reviews the proposal before it replaces the goal text.
function optimizeUi({q,request,state,notice,busy,applyGoal}) {
 let waiting=null,shown=null;
 // GitHub #273 (DD-208): a rewrite in progress shows its elapsed time and is followed again after the page reopens.
 const refresh=()=>{q('optimize-button').disabled=!!waiting||busy()||!q('goal').value.trim();if(typeof AC_PRESS!=='undefined')AC_PRESS.hold(q('optimize-button'),!!waiting);if(waiting){const s=Math.max(0,Math.floor((Date.now()-waiting.at)/1000));q('optimize-status').textContent=acMessage('startup.optimize_working')+' '+Math.floor(s/60)+':'+String(s%60).padStart(2,'0');}};
 function fill(list,title,items){title.hidden=!items.length;list.hidden=!items.length;list.replaceChildren(...items.map(text=>{const li=document.createElement('li');li.textContent=text;return li;}));}
 function show(proposal){shown=proposal;q('optimize-goal').textContent=proposal.optimized;fill(q('optimize-changes'),q('optimize-changes-title'),proposal.changes||[]);fill(q('optimize-questions'),q('optimize-questions-title'),proposal.questions||[]);q('optimize').showModal();}
 function done(text){waiting=null;q('optimize-status').textContent=text||'';refresh();}
 async function start(){if(waiting||!q('goal').value.trim())return;waiting={at:Date.now()};refresh();try{const r=await request({kind:'optimize_goal',revision:state().revision,goal:q('goal').value});waiting={job:r.id,at:waiting.at};}catch(e){done('');notice(e.message,true);}}
 function update(){const s=state();if(!waiting&&s?.job?.kind==='optimize'&&s.job.state==='running')waiting={job:s.job.id,at:s.job.started_ms||Date.now()};refresh();if(!waiting?.job||!s?.job||s.job.id!==waiting.job)return;const job=s.job;
  if(job.state==='completed'&&s.optimize?.job===job.id){done('');show(s.optimize.proposal);}
  else if(['failed','interrupted','stopped'].includes(job.state)){done('');notice(acMessage('startup.optimize_error',{value1:job.error||job.state}),true);}}
 q('optimize-button').onclick=start;
 q('optimize-use').onclick=()=>{if(shown)applyGoal(shown.optimized);shown=null;q('optimize').close();};
 q('optimize-keep').onclick=()=>{shown=null;q('optimize').close();};
 return {update,proposal:()=>shown};
}
