(async()=>{
 const checks=[],check=(name,pass)=>checks.push({name,pass:!!pass});
 check('#997: completed Goal notice is present',!!document.getElementById('completed-goal-notice'));
 if(typeof applyCompletedGoalPreview==='function'){
 const first=document.querySelector('.tabs [data-step]');
 first.dispatchEvent(new KeyboardEvent('keydown',{key:'Home',bubbles:true}));check('#997: Home opens step 1',i===0);
 first.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',bubbles:true}));check('#997: arrow opens step 2',i===1);
 document.querySelectorAll('.tabs [data-step]')[1].dispatchEvent(new KeyboardEvent('keydown',{key:'End',bubbles:true}));check('#997: End opens step 6',i===7);

 for(const index of [0,1,2,3,6,7]){
  document.querySelectorAll('.tabs [data-step]')[[0,1,2,3,6,7].indexOf(index)].click();
  check('#997: clicking the stage opens its archived content',i===index);
  check('#997: step '+index+' keeps the completed Goal locked',document.querySelector('.win').dataset.goalReadonly==='true');
  const mutations=[...document.querySelectorAll('[data-completed-lock]')];
  check('#997: step '+index+' disables mutations with a reason',mutations.length>10&&mutations.every(e=>e.disabled&&e.title.includes('새 Goal')));
  check('#997: step '+index+' keeps navigation and chat available',!document.getElementById('coordinator-chat-button').disabled&&[...document.querySelectorAll('[data-step]')].every(e=>e.getAttribute('aria-disabled')!=='true'));
 }
 i=3;show();openRawLogs();check('#997: raw logs open',!document.getElementById('preview-raw-logs').hidden);
 openTaskInspector();check('#997: details retain quota refresh while assignment is disabled',!document.querySelector('[data-detail-quota]').disabled&&document.querySelector('[data-detail-assign]').disabled);setTaskInspector(false);
 document.getElementById('coordinator-chat-button').click();
 const input=document.getElementById('chat-preview-input');input.value='이 목표를 수정해줘';
 document.getElementById('chat-preview-form').dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));
 check('#997: chat answers without changing the completed Goal',selectedGoal().state==='completed'&&document.getElementById('chat-preview').textContent.includes('새 Goal'));
 closeSidePanel('coordinator-chat-panel');
 i=7;show();document.querySelector('.file-diff-toggle').click();check('#997: diff remains viewable',!!document.querySelector('.file-diff-toggle[aria-expanded="true"]'));
 document.getElementById('review-main-action').click();check('#997: completed results still open',document.getElementById('goal-result-dialog').open);document.getElementById('goal-result-dialog').close();
 openWorkspace();check('#997: delete and New Goal remain available',!!document.querySelector('.goal-trash:not(:disabled)')&&!document.getElementById('overview-new-goal').disabled);closeSidePanel('workspace-overview');
 document.getElementById('completed-goal-new').click();check('#997: New Goal restores editable inputs',selectedGoalId==='draft'&&!document.getElementById('goal-entry').disabled&&document.querySelector('.win').dataset.goalReadonly==='false');
 selectedGoalId='rotation';i=7;show();
 }
 document.documentElement.dataset.interactionResult=JSON.stringify({theme:document.documentElement.dataset.theme,width:innerWidth,checks});
})();
