// GitHub #828: each agent card counts its sessions; the dialog lists them with state, Goal/task and times,
// stops a running one (its run) and cleans up finished, failed or stale ones after a confirmation.
window.acSessions=(()=>{
 const $=id=>document.getElementById('start-agent-sessions-'+id),dialog=()=>document.getElementById('start-agent-sessions');
 const STATE={running:'sessions.state_running',waiting:'sessions.state_waiting',done:'sessions.state_done',failed:'sessions.state_failed',stale:'sessions.state_stale',stopped:'sessions.state_stopped'};
 const finished=s=>['done','failed','stale','stopped'].includes(s.state),active=s=>s.state==='running'||s.state==='waiting';
 let all={},agent=null,label=id=>id,request=null,pending=null,soon;
 const time=ms=>{const d=new Date(ms);return d.toDateString()===new Date().toDateString()?d.toLocaleTimeString([],{hour:'2-digit',minute:'2-digit'}):d.toLocaleString([],{month:'numeric',day:'numeric',hour:'2-digit',minute:'2-digit'});};
 function paintChips(){
  for(const button of document.querySelectorAll('[data-sessions]')){
   const list=all[button.dataset.sessions]||[],running=list.filter(active).length,count=document.createElement('span');
   count.className='session-count';count.textContent=String(list.length);
   button.replaceChildren(acMessage('sessions.chip'),count);button.classList.toggle('has-running',running>0);
   button.setAttribute('aria-label',acMessage('sessions.chip_label',{value1:label(button.dataset.sessions),value2:list.length,value3:running}));
  }
 }
 function status(text){$('status').textContent=text||'';$('status').hidden=!text;}
 function paint(){
  const list=all[agent]||[],n=state=>list.filter(s=>s.state===state).length;
  $('title').textContent=acMessage('sessions.title',{value1:label(agent)});
  $('summary').textContent=list.length?acMessage('sessions.summary',{value1:n('running'),value2:n('waiting'),value3:n('done')+n('stopped'),value4:n('failed'),value5:n('stale')}):'';
  const rows=$('list');rows.replaceChildren();
  if(!list.length){const empty=document.createElement('p');empty.className='panel-note';empty.textContent=acMessage('sessions.empty');rows.append(empty);}
  for(const s of list){
   const row=document.createElement('div'),badge=document.createElement('span'),info=document.createElement('div'),task=document.createElement('strong'),meta=document.createElement('small'),action=document.createElement('button');
   row.className='session-row';row.setAttribute('role','listitem');row.dataset.session=s.run+'/'+s.task;
   badge.className='session-state '+s.state;badge.textContent=acMessage(STATE[s.state]);
   task.textContent=s.label;task.title=s.label;
   meta.textContent=acMessage('sessions.meta',{value1:s.goal,value2:time(s.started_ms),value3:time(s.last_ms)})+(s.note?' · '+s.note:'');meta.title=meta.textContent;
   info.append(task,meta);action.type='button';action.className='quiet';
   if(finished(s)){action.textContent=acMessage('sessions.clean');action.title=s.cleanable?acMessage('sessions.clean_title'):acMessage('native.session_run_still_active');action.disabled=!s.cleanable;action.onclick=()=>ask([s]);}
   else{action.textContent=acMessage('sessions.stop');action.title=acMessage('sessions.stop_title');action.disabled=!s.stoppable;action.onclick=()=>act({kind:'stop_session',run:s.run});}
   row.append(badge,info,action);rows.append(row);
  }
  const cleanable=list.filter(s=>finished(s)&&s.cleanable).length,clean=$('clean-finished');
  clean.disabled=!cleanable;clean.textContent=cleanable?acMessage('sessions.clean_finished_count',{value1:cleanable}):acMessage('sessions.clean_finished');
 }
 function ask(list){
  pending=list.map(s=>[s.run,s.task]);
  $('confirm-text').textContent=list.length===1?acMessage('sessions.confirm_one'):acMessage('sessions.confirm_many',{value1:list.length});
  $('confirm').hidden=false;$('confirm-cancel').focus();
 }
 async function act(packet){
  status('');
  try{all=await request(packet);}catch(e){status(String(e?.message||e));}
  pending=null;$('confirm').hidden=true;paintChips();if(dialog().open)paint();
 }
 async function refresh(){if(!request)return;try{all=await request({kind:'agent_sessions'});paintChips();if(dialog().open)paint();}catch{}}
 $('confirm-cancel').onclick=()=>{pending=null;$('confirm').hidden=true;$('clean-finished').focus();};
 $('confirm-ok').onclick=()=>act({kind:'clean_sessions',sessions:pending});
 $('clean-finished').onclick=()=>ask((all[agent]||[]).filter(s=>finished(s)&&s.cleanable));
 $('close').onclick=()=>dialog().close();
 return{
  // `labelOf` names an agent id; `send` is the startup request bridge.
  chip(id,send,labelOf){request=send;label=labelOf;const b=document.createElement('button');b.type='button';b.className='fact agent-sessions';b.dataset.sessions=id;b.setAttribute('aria-haspopup','dialog');b.onclick=()=>this.open(id);clearTimeout(soon);soon=setTimeout(refresh,0);return b;},
  refresh,paintChips,
  open(id){agent=id;pending=null;$('confirm').hidden=true;status('');paint();if(!dialog().open)dialog().showModal();refresh();}
 };
})();
