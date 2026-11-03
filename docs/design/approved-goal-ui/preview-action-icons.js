// Action icons follow button state changes; no polling or extra runtime dependency.
const actionIconPaths={list:'M8 6h12M8 12h12M8 18h12M3 6h1M3 12h1M3 18h1',save:'M4 3h13l3 3v15H4z M8 3v6h8V3 M8 21v-7h8v7',stop:'M6 6h12v12H6z',play:'M8 5l11 7-11 7z',plus:'M12 5v14M5 12h14',refresh:'M20 7v5h-5M4 17v-5h5 M6 6a8 8 0 0 1 13 4M18 18a8 8 0 0 1-13-4',check:'M5 12l4 4L19 6',reject:'M6 6l12 12M18 6L6 18',send:'M3 3l18 9-18 9 4-9z M7 12h14',folder:'M3 7V4h7l2 3h9v13H3z',review:'M14 2H5v20h14V7z M14 2v6h5 M8 12h7M8 16h5',link:'M10 13a5 5 0 0 0 7 0l3-3a5 5 0 0 0-7-7l-2 2 M14 11a5 5 0 0 0-7 0l-3 3a5 5 0 0 0 7 7l2-2',back:'M10 5l-7 7 7 7M3 12h18',sliders:'M4 7h16M4 17h16 M8 4v6M16 14v6',person:'M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0 M4 21v-2a8 8 0 0 1 16 0v2',model:'M6 6h12v12H6z M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4',fallback:'M4 7h15l-4-4M20 17H5l4 4',next:'M5 12h14M13 6l6 6-6 6'};
function updateActionIcon(button){
 if(button.closest('.ttabs')||button.querySelector('svg'))return;
 let label=button.textContent.trim(),icon='next';
 const field=button.dataset.assignmentField;
 if(button.classList.contains('assignment-choice'))icon=field==='fallback'?'fallback':field==='model'||field==='fallbackModel'||/모델|model/i.test(button.getAttribute('aria-label')||'')?'model':field==='agent'||/에이전트|계정|종류|agent|provider/i.test(button.getAttribute('aria-label')||'')?'person':'sliders';
 else{
  label=label.replace(/^[■✎+]\s*/, '');
  const rules=[[/Goal 목록|목표 목록/,'list'],[/중지|Stop/,'stop'],[/저장|적용|프로필 사용|Save|Apply/,'save'],[/추가|새 Goal|Add/,'plus'],[/다시|재생성|Regenerate/,'refresh'],[/승인|허용|Approve|Allow/,'check'],[/거절|거부|Reject|Deny/,'reject'],[/지시 보내기|보내기|Send/,'send'],[/찾아보기|폴더|Browse/,'folder'],[/검토|변경 열|Review/,'review'],[/Jira|Confluence|가져오기|Import/,'link'],[/취소|Cancel/,'back'],[/최적화|Optimize/,'sliders'],[/재개|계속|Resume/,'play']];
  icon=rules.find(([pattern])=>pattern.test(label))?.[1]||'next';
  if(label!==button.textContent.trim())button.textContent=label;
 }
 const image='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="'+actionIconPaths[icon]+'"/></svg>';
 button.dataset.actionIcon=icon;button.style.setProperty('--action-icon','url("data:image/svg+xml,'+encodeURIComponent(image)+'")');
}
const actionRoot=document.querySelector('.main');
document.querySelectorAll('.card button,button.btn,.assignment-choice').forEach(updateActionIcon);
const actionObserver=new MutationObserver(records=>{
 const buttons=new Set();for(const record of records){const target=record.target.nodeType===Node.ELEMENT_NODE?record.target:record.target.parentElement;const button=target.closest?.('button');if(button&&(button.closest('.card')||button.matches('.btn,.assignment-choice')))buttons.add(button);for(const node of record.addedNodes)if(node.nodeType===Node.ELEMENT_NODE)node.querySelectorAll?.('button').forEach(item=>{if(item.closest('.card'))buttons.add(item);});}
 buttons.forEach(updateActionIcon);
});
actionObserver.observe(actionRoot,{childList:true,characterData:true,subtree:true});document.querySelectorAll('dialog').forEach(dialog=>actionObserver.observe(dialog,{childList:true,characterData:true,subtree:true}));
