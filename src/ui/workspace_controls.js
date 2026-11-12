// #905/#906: shared Workspace save feedback, without changing backend validation.
function workspaceUi({q,state,draft,sources,save,notice,showStep,busy}) {
 let savedGoal=null,badRepository=null;
 const invalid=(id,bad)=>{const node=q(id);node.setAttribute('aria-invalid',String(bad));const choice=node.closest('.ac-selection-control')?.querySelector('.ac-choice-trigger');choice?.setAttribute('aria-invalid',String(bad));};
 function pending(){
  const current=sources.referenceState(),s=state(),goal=draft()?.goal||'';
  return current.ready&&current.goal===goal&&current.references.length>0&&
   (goal!==s?.config.goal||!s?.imports||current.references.some(ref=>!s.imports.selected.includes(ref)));
 }
 function render(){
  const config=draft();if(!config)return;
  const profile=config.profiles.find(p=>p.id===config.coordinator),name=q('name').value;
  invalid('name',!name.trim()||[...name].length>80||/[\x00-\x1f\x7f-\x9f]/.test(name));
  invalid('repository',!q('repository').value.trim()||badRepository===q('repository').value);
  invalid('goal',!q('goal').value.trim()||new TextEncoder().encode(q('goal').value).length>16384);
  invalid('coordinator',!profile);invalid('coordinator-model',!profile);
  invalid('mcp-preview',pending());q('save-only').disabled=busy();
  q('workspace-source-reminder').hidden=!(savedGoal===config.goal&&pending());
 }
 async function commit(advance){
  if(busy())return;
  if(await save(true)){savedGoal=draft().goal;render();notice(()=>acMessage(pending()?'startup.preview_before_planning':'startup.workspace_saved_locally'));if(advance)showStep('team',true);}
 }
 function failed(message){
  if(['native.project_directory_does_not_exist','native.project_path_must_be_a_directory','native.startup_artifacts_must_be_outside_repository'].some(key=>message===acMessage(key)))badRepository=q('repository').value;
  render();
 }
 q('save-only').onclick=()=>commit(false);q('save').onclick=()=>commit(true);
 q('repository').addEventListener('input',()=>{badRepository=null;});
 return {render,failed};
}
