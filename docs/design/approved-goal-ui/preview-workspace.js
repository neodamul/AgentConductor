// #905/#906: workspace save stays in place; missing inputs retain the reference style.
(() => {
 const q=id=>document.getElementById(id), card=document.querySelector('section.card[data-show="j e"]');
 const fields=[document.querySelector('input[aria-label="워크스페이스 이름"]'),document.querySelector('input[aria-label="작업 디렉터리"]'),q('goal-entry')];
 const preview=document.createElement('button');preview.type='button';preview.className='btn';preview.id='workspace-preview-documents';preview.textContent='문서 미리보기';card.querySelector('.goal-import-actions').append(preview);updateActionIcon(preview);
 window.previewWorkspaceApplied=false;
 window.previewWorkspaceValidity=()=>{for(const field of fields)field.setAttribute('aria-invalid',String(!field.value.trim()));for(const button of card.querySelectorAll('[data-assignment-target="coordinator"]'))button.setAttribute('aria-invalid',String(!card.querySelector('[data-assignment-target="coordinator"][data-assignment-field="agent"] span').textContent.trim()));preview.setAttribute('aria-invalid',String(/\b(?:jira|confluence):/i.test(q('goal-entry').value)&&!previewWorkspaceApplied));};
 fields.forEach(field=>field.addEventListener('input',previewWorkspaceValidity));
 const note=document.createElement('p');note.id='workspace-source-reminder';note.className='panel-note';note.hidden=true;note.setAttribute('role','status');q('goal-entry').parentElement.after(note);
 q('workspace-save-only').onclick=()=>{saveWorkspacePreview(false);previewWorkspaceValidity();note.textContent='참조 문서를 계획에 포함하려면 문서 미리보기에서 검토하고 적용하세요. 저장은 완료되었습니다.';note.hidden=!/\b(?:jira|confluence):/i.test(q('goal-entry').value);card.dataset.saved='true';};
 previewWorkspaceValidity();
})();
