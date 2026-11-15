// #997 design scenario only: a completed Goal is an immutable, browsable archive.
if(new URLSearchParams(location.search).get('goal')==='completed'){
 const reason='완료된 Goal은 읽기 전용입니다. 변경하려면 새 Goal을 추가하세요.';
 const notice=document.createElement('div');notice.id='completed-goal-notice';notice.className='runmeta';notice.setAttribute('role','status');
 const label=document.createElement('span');label.textContent='✓ 완료 · 읽기 전용';label.title=reason;
 const hint=document.createElement('span');hint.textContent='변경은 새 Goal에서';hint.title=reason;
 const add=document.createElement('button');add.type='button';add.className='btn q';add.id='completed-goal-new';add.textContent='+ 새 Goal';add.onclick=()=>startNewGoal();
 notice.append(label,hint,add);
 const info=document.querySelector('.goal-info-content');info.append(notice);
 const locked=new Map(),originalText=new Map();
 function setText(node,text){if(!node)return;if(!originalText.has(node))originalText.set(node,node.textContent);node.textContent=text;}
 function lock(node){
  if(!locked.has(node))locked.set(node,{disabled:node.disabled,title:node.title});
  node.disabled=true;node.title=reason;node.dataset.completedLock='true';node.setAttribute('aria-describedby','completed-goal-notice');
 }
 function applyCompletedGoalPreview(){
  const readonly=selectedGoalId!=='draft'&&selectedGoal().state==='completed'&&!selectedGoal().nextPhase;
  document.querySelector('.win').dataset.goalReadonly=String(readonly);notice.hidden=!readonly;notice.style.display=readonly?'flex':'none';
  for(const [node,old] of locked){node.disabled=old.disabled;node.title=old.title;delete node.dataset.completedLock;node.removeAttribute('aria-describedby');}locked.clear();
  for(const [node,text] of originalText)node.textContent=text;originalText.clear();
  if(!readonly)return;
  if(views[i][0]==='j')document.getElementById('goal-entry').value=selectedGoal().description;
  // Workspace/global settings remain separate; freeze only this archived Goal's controls.
  document.querySelectorAll('.main input,.main textarea,.main select,[data-assignment-target],#task-inspector-body input,#task-inspector-body select,[data-detail-assign],[data-detail-model-save],[data-detail-schedule]').forEach(lock);
  document.querySelectorAll('.card .dir button').forEach(node=>{if(node.id!=='review-main-action'&&node.id!=='workspace-new-goal')lock(node);});
  document.querySelectorAll('#optimize-preview-open,#source-reference,#workspace-preview-documents,[data-role-defaults]').forEach(lock);
  document.querySelectorAll('.goal-info-content>.runmeta:not(#completed-goal-notice)').forEach(node=>node.style.display='none');
  notice.hidden=false;notice.style.display='flex';
  document.querySelectorAll('.tabs [data-step]').forEach(tab=>{
   tab.setAttribute('aria-selected',String(tab.classList.contains('cur')));tab.tabIndex=tab.classList.contains('cur')?0:-1;
   tab.classList.toggle('done',!tab.classList.contains('cur'));tab.querySelector('.num').textContent=tab.classList.contains('cur')?tab.dataset.step:'✓';
  });
  document.querySelectorAll('.goal-info>.pill').forEach(node=>node.style.display='none');
  updateAgentStates('d');
  document.getElementById('outline-completed').textContent='작업 3/3 완료';
  document.querySelectorAll('.outline-state-badge').forEach(node=>{node.dataset.state='completed';node.textContent='완료';});
  document.querySelectorAll('.outline-agent').forEach(node=>node.dataset.state='completed');
  document.querySelectorAll('[data-team-brief]').forEach(button=>button.disabled=false);
  setText(document.getElementById('plan-heading'),'완료된 계획');
  setText(document.getElementById('plan-sub'),'계획과 작업 지시를 펼쳐 확인할 수 있습니다. 변경은 새 Goal에서 진행하세요.');
  setText(document.getElementById('plan-approve-note'),reason);
  setText(document.querySelector('.card[data-show="b"] .pill'),'완료');
  setText(document.querySelector('.card[data-show="b"] .task-activity-content .note'),reason);
  setText(document.querySelector('.card[data-show="f"] h2'),'최종 검증 결과');
  setText(document.querySelector('.card[data-show="f"] .sub'),'세 작업의 통합 검증이 완료되었습니다. 기록과 로그를 확인할 수 있습니다.');
  document.querySelectorAll('.card[data-show="f"] .row .s').forEach(node=>{if(/실행 중|대기/.test(node.textContent))setText(node,'통과 · 4초');});
  setText(document.querySelector('#coordinator-chat-panel .panel-note'),'Auth Service · 완료된 Goal · 질문과 기록 조회 가능');
  setText(document.getElementById('workspace-current-goal-state'),'완료 · 읽기 전용');
  setText(document.querySelector('.card[data-show="c"] .dir .note'),'완료된 기록입니다. 변경 파일과 검증 결과를 확인하거나 새 Goal을 추가하세요.');
  setText(document.querySelector('.card[data-show="c"] .st:last-child .v'),'병합 완료');
 }
 window.applyCompletedGoalPreview=applyCompletedGoalPreview;
 const originalShow=show;show=function(){originalShow();applyCompletedGoalPreview();};
 const originalInspector=openTaskInspector;openTaskInspector=function(){originalInspector();applyCompletedGoalPreview();};
 const chatForm=document.getElementById('chat-preview-form'),oldChat=chatForm.onsubmit;
 chatForm.onsubmit=event=>{
  if(document.querySelector('.win').dataset.goalReadonly!=='true')return oldChat(event);
  event.preventDefault();const input=document.getElementById('chat-preview-input');if(!input.value.trim())return;
  const question=document.createElement('p'),answer=document.createElement('p');question.textContent=input.value.trim();
  answer.textContent='코디네이터 · 완료된 Goal의 기록을 바탕으로 질문에 답할 수 있습니다. 변경 요청은 새 Goal로 진행하세요. 이 Goal의 계획과 결과는 수정하지 않습니다.';
  document.getElementById('chat-preview').append(question,answer);input.value='';
 };
 const stageViews=[0,1,2,3,6,7],stageTabs=[...document.querySelectorAll('.tabs [data-step]')];
 stageTabs.forEach((tab,index)=>{
  tab.setAttribute('role','tab');tab.style.cursor='pointer';tab.title='저장된 '+tab.querySelector('.tab-label').textContent+' 조회';
  tab.addEventListener('click',()=>{workspaceVisible=false;i=stageViews[index];show();});
  tab.addEventListener('keydown',event=>{
   if(['Enter',' '].includes(event.key)){event.preventDefault();tab.click();}
   if(['ArrowLeft','ArrowRight','Home','End'].includes(event.key)){
    event.preventDefault();const next=event.key==='Home'?0:event.key==='End'?5:(index+(event.key==='ArrowRight'?1:5))%6;
    stageTabs[next].click();stageTabs[next].focus();
   }
  });
 });
 const goal=selectedGoal();goal.state='completed';goal.step=7;goal.position='Phase 1/1 완료';goal.summary='main 반영 · 통합 테스트 3/3 통과';
 if(!location.hash||!/^#[0-7]$/.test(location.hash))i=7;
 show();
}
