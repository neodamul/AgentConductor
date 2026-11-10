// GitHub #392 / DD-222: the left rail has two tabs, Agents and Phases & tasks, on both screens. The tab strip
// and the agent panel (add button, quota, every agent card) are one pair of nodes that moves between the
// Workspace rail and the Agent Room rail like the stepper, so the cards keep all their functions.
window.acRail=(()=>{
 const $=id=>document.getElementById(id),names=['agents','tasks'];
 let tab='agents',marks;
 const onRun=()=>document.body.classList.contains('on-run');
 function paint(){
  for(const name of names){const t=$('rail-tab-'+name),on=name===tab;t.setAttribute('aria-selected',String(on));t.tabIndex=on?0:-1;}
  $('rail-pane-agents').hidden=tab!=='agents';
  $('rail-outline').hidden=onRun()||tab!=='tasks';
  $('rail-room').hidden=!onRun()||tab!=='tasks';
 }
 return{
  names,
  select(name){if(names.includes(name)){tab=name;paint();}},
  // Steps 1-2 open on Agents, steps 3-6 on Phases & tasks; the operator's own choice stands until the step changes.
  forStep(step){tab=step==='workspace'||step==='team'?'agents':'tasks';paint();},
  // Workspace screen: the plan under review in the mock's outline (current Phase and Task, review state, the Phase, one row per task).
  // `makeAvatar(agentId)` draws the agent mark; a row opens its task in the plan card.
  outline(plan,makeAvatar){
   const box=$('rail-outline-body'),tasks=plan?.tasks||[];
   $('rail-outline-empty').hidden=tasks.length>0;
   if(!tasks.length){box.replaceChildren();return;}
   const el=(tag,cls,text)=>{const n=document.createElement(tag);if(cls)n.className=cls;if(text!=null)n.textContent=text;return n;};
   const line=prompt=>String(prompt||'').split('\n').map(l=>l.trim().replace(/^[-*]\s+/,'')).find(l=>l&&!/:$/.test(l))||'';
   const status=el('div','outline-run-status');status.append(el('strong',null,acMessage('startup.rail_plan_review')),el('span',null,acMessage('startup.rail_not_started')));
   const tree=el('details','phase-tree'),head=el('summary'),phase=el('span','phase-name','1. '+line(plan.goal).slice(0,60)),count=el('span','phase-count',acMessage('room.rail_current_phase'));tree.open=true;head.append(phase,count);tree.append(head);
   tasks.forEach((task,index)=>{
    const row=el('button','outline-task'),mark=makeAvatar?.(task.agent)||el('span',null,'?'),copy=el('span','outline-copy'),top=el('span','outline-task-heading'),title=el('strong'),badge=el('span','outline-state-badge',acMessage('startup.rail_task_planned'));
    row.type='button';mark.classList?.add('outline-agent','st-yellow');badge.dataset.state='waiting';
    title.append(el('code','task-number',task.id),' ',line(task.prompt).slice(0,120));top.append(title,badge);copy.append(top,el('small','task-description',(task.paths||[]).slice(0,2).join(', ')||'—'));
    const chevron=el('span','outline-open-icon');chevron.setAttribute('aria-hidden','true');chevron.innerHTML='<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M9 5l7 7-7 7"/></svg>';
    row.setAttribute('aria-pressed',String(index===0));row.append(mark,copy,chevron);
    row.onclick=()=>{for(const other of tree.querySelectorAll('.outline-task'))other.setAttribute('aria-pressed',String(other===row));const card=document.querySelectorAll('#start-tasks details.startup-task')[index];if(card){card.open=true;card.scrollIntoView({block:'nearest'});}};
    tree.append(row);
   });
   box.replaceChildren(el('div','sub',acMessage('room.rail_current',{value1:1,value2:1,value3:1,value4:tasks.length})),status,tree,el('div','outline-note',acMessage('room.rail_note')));
  },
  // `host` is the Room's rail while a run is open (null brings the pair back to the Workspace rail).
  move(host){
   const tabs=$('rail-tabs'),pane=$('rail-pane-agents');
   if(!marks){marks=[document.createComment('rail-tabs'),document.createComment('rail-agents')];tabs.before(marks[0]);pane.before(marks[1]);}
   if(host){host.prepend(tabs,pane);}else{marks[0].after(tabs);marks[1].after(pane);}
   paint();
  },
  bind(){
   for(const name of names){
    const t=$('rail-tab-'+name);
    t.onclick=()=>this.select(name);
    t.onkeydown=event=>{const to=event.key==='ArrowRight'||event.key==='ArrowLeft'?names[(names.indexOf(name)+1)%names.length]:event.key==='Home'?names[0]:event.key==='End'?names[names.length-1]:null;if(!to)return;event.preventDefault();this.select(to);$('rail-tab-'+to).focus();};
   }
  }
 };
})();
