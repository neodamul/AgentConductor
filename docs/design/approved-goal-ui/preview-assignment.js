// Fixture catalogs only. Production uses each CLI's actual catalog/capabilities.
const assignmentCatalog={claude:['sonnet','opus'],codex:['등록 모델 A','등록 모델 B'],'custom-cli':[]};
const agentDisplay={claude:'Claude Code',codex:'Codex','custom-cli':'custom-cli'};
const roleAssignments=[{role:'계획과 검증',agent:'custom-cli',model:'',fallback:'',fallbackModel:''},{role:'개발',agent:'claude',model:'sonnet',fallback:'codex',fallbackModel:''},{role:'테스트',agent:'codex',model:'',fallback:'custom-cli',fallbackModel:''},{role:'문서',agent:'custom-cli',model:'',fallback:'',fallbackModel:''}];
let workspaceCoordinator={agent:'custom-cli',model:''},assignmentTarget=null,assignmentField=null,assignmentAnchor=null;
function assignmentValue(target){return target==='coordinator'?workspaceCoordinator:roleAssignments[Number(target.split('-')[1])];}
function refreshAssignmentChoices(){
 document.querySelectorAll('[data-assignment-target]').forEach(button=>{
  const value=assignmentValue(button.dataset.assignmentTarget),field=button.dataset.assignmentField;
  const text=field==='agent'?agentDisplay[value.agent]:field==='fallback'?(agentDisplay[value.fallback]||'지정 안 함'):value[field]||'CLI 기본 모델';
  button.querySelector('span').textContent=text;button.title=text;
  button.disabled=field==='fallbackModel'&&!value.fallback;
 });
}
function chooseAssignment(value){
 const assignment=assignmentValue(assignmentTarget);
 assignment[assignmentField]=value;
 if(assignmentField==='agent'){assignment.model='';if(assignment.fallback===value){assignment.fallback='';assignment.fallbackModel='';}}
 if(assignmentField==='fallback')assignment.fallbackModel='';
 if(assignmentTarget==='coordinator')Object.assign(roleAssignments[0],{agent:assignment.agent,model:assignment.model});
 refreshAssignmentChoices();
}
function openAssignmentPicker(target,field,anchor){
 field=field||(target==='coordinator'?'agent':'model');assignmentTarget=target;assignmentField=field;
 anchor=anchor||document.querySelector('[data-assignment-target="'+target+'"][data-assignment-field="'+field+'"]');
 const assignment=assignmentValue(target),isAgent=field==='agent'||field==='fallback',provider=field==='fallbackModel'?assignment.fallback:assignment.agent;
 const names={agent:'에이전트',model:'모델',fallback:'Fallback 에이전트',fallbackModel:'Fallback 모델'};
 let choices=isAgent?Object.entries(agentDisplay):[['','CLI 기본 모델'],...(assignmentCatalog[provider]||[]).map(model=>[model,model]),['__custom','모델 ID 직접 입력…']];
 if(field==='fallback')choices=[['','지정 안 함'],...choices.filter(([id])=>id!==assignment.agent)];
 openSelectionMenu({anchor,title:(target==='coordinator'?'코디네이터':assignment.role)+' · '+names[field],value:assignment[field],options:choices.map(([value,label])=>({value,label,custom:value==='__custom'})),note:isAgent?'등록된 CLI 중에서 선택합니다.':provider==='custom-cli'?'이 CLI는 모델 목록 조회를 지원하지 않습니다 · CLI 기본값 또는 직접 입력':'CLI가 제공한 모델 목록',onSelect:chooseAssignment});
}
document.querySelectorAll('[data-assignment-target]').forEach(button=>button.onclick=()=>openAssignmentPicker(button.dataset.assignmentTarget,button.dataset.assignmentField,button));
refreshAssignmentChoices();
const initialPicker=new URLSearchParams(location.search).get('picker');
if(initialPicker==='coordinator'||/^role-[0-3]$/.test(initialPicker||''))setTimeout(()=>openAssignmentPicker(initialPicker),100);
// Apply agent role defaults: Claude Code verifies, Codex develops and documents, the custom CLI tests; each keeps the other as fallback.
function applyRoleDefaults(){
 const defaults={'계획과 검증':['claude','codex'],'개발':['codex','claude'],'테스트':['custom-cli','claude'],'문서':['codex','claude']};
 roleAssignments.forEach(item=>{const [agent,fallback]=defaults[item.role]||['claude',''];Object.assign(item,{agent,model:'',fallback,fallbackModel:''});});
 Object.assign(workspaceCoordinator,{agent:roleAssignments[0].agent,model:''});
 refreshAssignmentChoices();
}
