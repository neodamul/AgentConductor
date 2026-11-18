// Profile editor reference: fields mirror the current main UI; no executable access.
const profileFixtures={claude:{provider:'claude',id:'claude',executable:'claude',args:['-p','--output-format','stream-json'],input:'stdin',sessions:3,minutes:5,fallback:'codex',fallbackModel:'',model:'claude-opus-5-5',effort:'high'},codex:{provider:'codex',id:'codex',executable:'codex',args:['exec','-','--json'],input:'stdin',sessions:3,minutes:5,fallback:'custom-cli',fallbackModel:''},'custom-cli':{provider:'custom',id:'custom-cli',executable:'custom-cli',args:[],input:'stdin',sessions:3,minutes:5,fallback:'',fallbackModel:''}};
let editingProfile=null,profileOpener=null;
const profileDialog=document.getElementById('profile-edit-dialog'),profileField=name=>document.getElementById('profile-'+name);
function refreshProfileFallbackModels(model=''){
 const fallback=profileField('fallback').value,select=profileField('fallback-model');select.replaceChildren();
 const options=[['','CLI 기본 모델'],...(assignmentCatalog[fallback]||[]).map(item=>[item,item]),['__custom','모델 ID 직접 입력…']];
 options.forEach(([value,label])=>{const option=document.createElement('option');option.value=value;option.textContent=label;select.append(option);});
 const known=options.some(([value])=>value===model);select.value=known?model:'__custom';select.disabled=!fallback;profileField('custom-model').value=known?'':model;syncProfileCustom();syncSelectionButtons();
}
// GitHub #1024: the agent's default model comes from its queried model list; Effort lists the levels, empty follows the task type.
function refreshProfileModels(agent,model=''){
 const select=profileField('model'),models=[...document.querySelectorAll('#model-list-'+CSS.escape(agent)+' > span:not(.more):not(.model-default)')].map(item=>item.textContent);select.replaceChildren();
 [['','CLI 기본 모델'],...models.map(item=>[item,item])].forEach(([value,label])=>{const option=document.createElement('option');option.value=value;option.textContent=label;select.append(option);});
 select.value=models.includes(model)?model:'';
}
function syncProfileCustom(){const custom=!!profileField('fallback').value&&profileField('fallback-model').value==='__custom';profileField('custom-model-label').hidden=!custom;profileField('custom-model').required=custom;}
function openProfileEditor(agent){
 editingProfile=agent;profileOpener=document.activeElement;
 // The add button opens the same dialog empty; the fixture is rebuilt each time so nothing carries over.
 if(agent==='new')profileFixtures.new={provider:'custom',id:'',executable:'',args:[],input:'stdin',sessions:1,minutes:5,fallback:'',fallbackModel:'',model:'',effort:''};
 const profile=profileFixtures[agent];
 document.getElementById('profile-edit-title').textContent=agent==='new'?'에이전트 추가':(agentDisplay[agent]||agent)+' · 편집';
 document.querySelector('#profile-edit-form button[type=submit]').textContent=agent==='new'?'에이전트 추가':'프로필 사용';
 for(const field of ['provider','id','executable','input','sessions'])profileField(field).value=profile[field];
 profileField('callback-label').hidden=profile.provider!=='claude';profileField('name-label').hidden=profile.provider!=='custom';profileField('name').value=profile.name||'';profileField('callback').checked=profile.callback!==false;profileField('advanced').open=false;profileField('quota-minutes').value=profile.minutes;profileField('args').value=JSON.stringify(profile.args);profileField('readonly-args').value=JSON.stringify(profile.readonlyArgs||[]);profileField('edit-error').textContent='';
 const fallback=profileField('fallback');fallback.replaceChildren();
 [['','지정 안 함'],...Object.entries(agentDisplay).filter(([id])=>id!==agent)].forEach(([value,label])=>{const option=document.createElement('option');option.value=value;option.textContent=label;fallback.append(option);});
 fallback.value=profile.fallback;refreshProfileFallbackModels(profile.fallbackModel);refreshProfileModels(agent,profile.model||'');profileField('effort').value=profile.effort||'';syncSelectionButtons();profileDialog.showModal();profileField('executable').focus({preventScroll:true});
}
profileField('fallback').onchange=()=>refreshProfileFallbackModels();profileField('fallback-model').onchange=syncProfileCustom;
profileField('edit-cancel').onclick=()=>profileDialog.close('cancel');
profileField('browse').onclick=()=>{profileField('edit-error').textContent='';profileField('executable').focus();};
document.getElementById('profile-edit-form').onsubmit=event=>{
 event.preventDefault();try{
  const args=JSON.parse(profileField('args').value||'[]'),readonlyArgs=JSON.parse(profileField('readonly-args').value||'[]');
  if(![args,readonlyArgs].every(value=>Array.isArray(value)&&value.every(item=>typeof item==='string')))throw new Error('실행 인자는 문자열 JSON 배열이어야 합니다.');
  const id=profileField('id').value.trim(),executable=profileField('executable').value.trim();if(!id||!executable)throw new Error('에이전트 ID와 실행파일을 입력하세요.');
  const fallbackModel=profileField('fallback-model').value==='__custom'?profileField('custom-model').value.trim():profileField('fallback-model').value;
  Object.assign(profileFixtures[editingProfile],{model:profileField('model').value,effort:profileField('effort').value,callback:profileField('callback').checked,provider:profileField('provider').value,id,executable,args,readonlyArgs,input:profileField('input').value,sessions:Number(profileField('sessions').value),minutes:Number(profileField('quota-minutes').value),fallback:profileField('fallback').value,fallbackModel,name:profileField('provider').value==='custom'?profileField('name').value.trim():''});
  // GitHub #1071: a custom CLI's display name replaces its id on the card and in the pickers.
  const shown=profileFixtures[editingProfile].name||id;if(profileFixtures[editingProfile].provider==='custom'&&editingProfile in agentDisplay){agentDisplay[editingProfile]=shown;document.querySelectorAll('[data-profile-edit="'+CSS.escape(editingProfile)+'"]').forEach(b=>{const n=b.closest('.agent')?.querySelector('.name');if(n&&n.firstChild?.nodeType===3)n.firstChild.textContent=shown;});}profileDialog.close('apply');
 }catch(error){profileField('edit-error').textContent=error.message;}
};
profileDialog.addEventListener('close',()=>{closeSelectionMenu();document.body.append(selectionMenu);if(profileOpener?.isConnected)profileOpener.focus({preventScroll:true});});
document.querySelectorAll('[data-profile-edit]').forEach(button=>button.onclick=()=>openProfileEditor(button.dataset.profileEdit));
document.getElementById('profile-add').onclick=()=>openProfileEditor('new');

profileField('provider').addEventListener('change',()=>{profileField('callback-label').hidden=profileField('provider').value!=='claude';profileField('name-label').hidden=profileField('provider').value!=='custom';});
