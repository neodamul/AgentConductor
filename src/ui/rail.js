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
  // Workspace screen: the plan under review, task by task, or a sentence saying it is not there yet.
  outline(plan){
   const box=$('rail-outline-body'),list=document.createElement('ol');
   list.className='rail-outline-list';
   for(const task of plan?.tasks||[]){
    const item=document.createElement('li'),id=document.createElement('strong'),text=document.createElement('span');
    id.textContent=task.id;text.textContent=String(task.prompt||'').split('\n')[0].slice(0,120);
    item.append(id,' ',text);list.append(item);
   }
   box.replaceChildren(...(list.children.length?[list]:[]));
   $('rail-outline-empty').hidden=list.children.length>0;
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
