// #997: one policy for archived Goal writes; no observers or background work.
// GitHub #1040: the reusable agent registry (cards, profile dialog, save_profiles) is not part of a
// Goal, so it stays manageable; the completed Goal keeps its own recorded assignments.
globalThis.acGoalReadOnly=(()=>{
 const startupWrites=new Set(['save','save_auto_merge','prepare','draft','read_plan','edit','start','import_settings','optimize_goal','mcp_apply','mcp_preview','create_slug','commit_baseline','ignore_os_files']);
 const locks=new WeakMap(),infoNodes=new WeakMap();
 const completed=s=>s?.run_state==='completed'||s?.report?.state==='completed'||!!s?.slugs?.some(g=>g.active&&g.progress?.stage==='completed');
 const reason=()=>acMessage('goal.readonly_reason');
 // GitHub #1039: other writers (timers, renders) keep assigning disabled/readOnly; while locked the
 // property is pinned so their writes are remembered for unlock instead of flipping the control.
 function native(node,key){for(let p=Object.getPrototypeOf(node);p;p=Object.getPrototypeOf(p)){const d=Object.getOwnPropertyDescriptor(p,key);if(d)return d;}}
 function pin(node,key,old){const d=native(node,key);if(!d?.set||Object.hasOwn(node,key))return;d.set.call(node,true);Object.defineProperty(node,key,{configurable:true,get:()=>true,set:v=>{old[key]=v;}});}
 function block(node,on){
  if(on){
   if(!locks.has(node))locks.set(node,{disabled:node.disabled,readOnly:node.readOnly,title:node.title});
   pin(node,node.matches('textarea,input:not([type=checkbox]):not([type=radio])')?'readOnly':'disabled',locks.get(node));
   node.dataset.goalReadonly='true';AC_I18N.bind(node,reason,'title');
  }else if(locks.has(node)){
   const old=locks.get(node);delete node.disabled;delete node.readOnly;if(old.disabled!==undefined)node.disabled=old.disabled;if(old.readOnly!==undefined)node.readOnly=old.readOnly;
   AC_I18N.bind(node,()=>old.title||'','title');delete node.dataset.goalReadonly;locks.delete(node);
  }
 }
 function apply(selectors,on){document.querySelectorAll(selectors).forEach(n=>block(n,on));}
 function info(selector,on){
  const box=document.querySelector(selector);if(!box)return;
  if(!on){if(box.firstElementChild?.dataset.completedLabel){AC_I18N.replace(box);delete box.dataset.completedInfo;}return;}
  let nodes=infoNodes.get(box);
  if(!nodes){
   const label=document.createElement('span');label.dataset.completedLabel='true';AC_I18N.bind(label,()=>acMessage('goal.readonly_label'));AC_I18N.bind(label,reason,'title');nodes=[label];
   if(document.getElementById('start-new-goal')){const add=document.createElement('button');add.type='button';add.className='quiet small-button';AC_I18N.bind(add,()=>acMessage('startup.goal_list_new'));add.onclick=()=>window.acNewGoal?.();nodes.push(add);}
   infoNodes.set(box,nodes);
  }
  if(box.firstChild!==nodes[0])AC_I18N.replace(box,...nodes);
  box.dataset.completedInfo='true';
 }
 const workspaceSelectors=[
 '#start-panel-workspace input','#start-panel-workspace textarea','#start-panel-workspace select','#start-panel-workspace .project-input button',
 '#start-panel-team input','#start-panel-team select','#start-panel-team button',
 '#start-tasks input','#start-tasks textarea','#start-tasks select','#start-tasks .task-remove','#start-task-tools button',
 '#start-save-only','#start-save','#start-discard','#start-prepare','#start-regenerate','#start-draft','#start-read-plan','#start-edit','#start-run','#start-auto-merge',
 '#start-optimize-button','#start-reference','#start-source-suggestions button','#start-mcp-preview','#start-mcp-apply',
 '#start-import button[value=confirm]','#start-source-add','#start-optimize-again','#start-optimize-use','#start-optimize-answers'
 ].join(',');
 const roomSelectors=['send-feedback','feedback','reassign','agent-select','set-model','model-select','set-effort','effort-select','schedule','resume-at','resume','recover','stop','approve','reject','merge','review-revise','review-deny','tool-decision-allow','tool-decision-deny','scope-review','scope-preview','scope-approve','scope-task','scope-paths','scope-test','fix-integration'].map(id=>'#'+id).join(',');
 return{
  completed,reason,
  blocks(s,packet){return completed(s)&&(packet.kind==='action'?packet.action?.kind!=='quota':startupWrites.has(packet.kind));},
  workspace(s){const on=completed(s);apply(workspaceSelectors,on);info('#start-goal-info .goal-info-content',on);},
  room(s){const on=completed(s);apply(roomSelectors+',#activity [data-goal-write],#tool-requests [data-goal-write]',on);info('#run-goal-info .goal-info-content',on);},
 };
})();
