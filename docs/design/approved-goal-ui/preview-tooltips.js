
document.getElementById('mock-theme').value=document.documentElement.dataset.theme;document.getElementById('mock-theme').onchange=event=>{document.documentElement.dataset.theme=event.target.value;};
const overflowTextSelector='.tab-label,.ttabs>*,.rail-tabs>button,.head h1,.cardhead h2,.outline-copy strong,.outline-copy small,.goal-list-row strong,.goal-selection-context>span:first-child,.runmeta>span,#task-inspector-context,#label,.workspace-home,.role-choice-row>.s,.role-choice-header>span';
function setFullTextTitle(element){const tab=element.closest('.tabs .tab');if(tab){const index=Number(tab.dataset.step)-1;element.title=(index<3?'준비':index<5?'실행':'마무리')+' · '+element.textContent.trim();}else element.title=element.textContent.trim();}
document.querySelectorAll(overflowTextSelector).forEach(setFullTextTitle);
document.addEventListener('pointerover',event=>{const element=event.target.closest(overflowTextSelector);if(element)setFullTextTitle(element);});

document.querySelectorAll('.main>.card .cardhead .eyebrow').forEach(label=>{label.textContent=label.textContent.replace(/^([1-6])단계 · /,(_,step)=>step+'단계 · '+(Number(step)<=3?'준비':Number(step)<=5?'실행':'마무리')+' · ');});
