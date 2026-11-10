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
  outline(plan,view,makeAvatar){
   const box=$('rail-outline-body'),tasks=plan?.tasks||[],phased=view?.phases?.length;
   $('rail-outline-empty').hidden=tasks.length>0||!!phased;
   if(!tasks.length&&!phased){box.replaceChildren();return;}
   const el=(tag,cls,text)=>{const n=document.createElement(tag);if(cls)n.className=cls;if(text!=null)n.textContent=text;return n;};
   const line=prompt=>String(prompt||'').split('\n').map(l=>l.trim().replace(/^[-*]\s+/,'')).find(l=>l&&!/:$/.test(l)&&!/^<\/?[\w-]+>$/.test(l))||'';
   const status=el('div','outline-run-status');const total=phased?view.phases.reduce((n,g)=>n+(g.tasks||[]).length,0):tasks.length;status.append(el('strong',null,view?.stale?acMessage('room.rail_done',{value1:total,value2:total}):acMessage('startup.rail_plan_review')),el('span',null,view?.stale?'':acMessage('startup.rail_not_started')));
   const chevron=()=>{const c=el('span','outline-open-icon');c.setAttribute('aria-hidden','true');c.innerHTML='<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M9 5l7 7-7 7"/></svg>';return c;};
   let number=0;const row=(task,index,tree,done,editable)=>{number++;
    const b=el('button','outline-task'),mark=makeAvatar?.(task.agent)||el('span',null,'?'),copy=el('span','outline-copy'),top=el('span','outline-task-heading'),title=el('strong'),badge=el('span','outline-state-badge',acMessage(done?'startup.completed_22a970':'startup.rail_task_planned'));
    b.type='button';mark.classList?.add('outline-agent',done?'st-green':'st-yellow');badge.dataset.state=done?'completed':'waiting';
    title.append(el('code','task-number',phased?'T'+number:task.id),' ',(line(task.prompt)||task.id).slice(0,120));top.append(title,badge);copy.append(top,el('small','task-description',(task.paths||[]).slice(0,2).join(', ')||'—'));
    b.setAttribute('aria-pressed',String(editable&&index===0));b.append(mark,copy,chevron());
    if(editable)b.onclick=()=>{for(const other of tree.querySelectorAll('.outline-task'))other.setAttribute('aria-pressed',String(other===b));const card=document.querySelectorAll('#start-tasks details.startup-task')[index];if(card){card.open=true;card.scrollIntoView({block:'nearest'});}};
    return b;};
   // GitHub #736: a phased Goal lists every Phase with its own tasks; the Phase under review holds the editable plan.
   const groups=phased?view.phases:[{name:line(plan.goal),current:true,tasks,done:[]}];
   const trees=groups.map((g,gi)=>{const tree=el('details','phase-tree'),head=el('summary');tree.open=!!g.current||(view?.stale&&gi===groups.length-1);
    head.append(el('span','phase-name',(gi+1)+'. '+String(g.name||'').slice(0,60)),el('span','phase-count',acMessage(g.stage==='completed'?'startup.completed_22a970':g.current?'room.rail_current_phase':'startup.planned')));tree.append(head);const raw=String(g.brief||'').trim(),brief=/^tasks?:/i.test(raw)?'':line(raw);if(brief)tree.append(el('p','phase-brief',brief.slice(0,120)));
    (g.tasks||[]).forEach((t,i)=>tree.append(row(t,i,tree,g.stage==='completed'||(g.done||[]).includes(t.id),!!g.current&&!view?.stale)));return tree;});
   const cur=view?.stale?groups.length-1:Math.max(0,groups.findIndex(g=>g.current)),curTasks=groups[cur]?.tasks?.length||0;
   box.replaceChildren(el('div','sub',acMessage('room.rail_current',{value1:cur+1,value2:groups.length,value3:view?.stale?curTasks:Math.min(1,curTasks),value4:curTasks})),status,...trees,el('div','outline-note',acMessage('room.rail_note')));
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
