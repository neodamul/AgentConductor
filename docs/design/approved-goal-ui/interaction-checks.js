// Design-reference browser regression checks; never exercise real CLI or disk actions.
(async()=>{
 const checks=[];
 function check(name,pass){checks.push({name,pass:!!pass});}
 const wait=()=>new Promise(resolve=>setTimeout(resolve,260));
 async function closeDialog(dialog,value){const closed=new Promise(resolve=>dialog.addEventListener('close',resolve,{once:true}));dialog.close(value);await closed;}
 const initialTheme=document.documentElement.dataset.theme;
 const inspector=document.getElementById('task-inspector-panel');
 check('right inspector defaults closed',inspector.hidden);
 check('right inspector has no close button',!inspector.querySelector('.panel-close,[data-panel-close]'));
 check('no body Details tab',![...document.querySelectorAll('.ttabs>*')].some(el=>el.textContent==='상세'));
 for(let index=0;index<8;index++){
  i=index;show();
  const card=[...document.querySelectorAll('.main>.card')].find(el=>getComputedStyle(el).display!=='none');
  const body=card.querySelector('.card-body'),head=card.querySelector('.cardhead'),footer=card.querySelector('.dir');
  const spacer=document.createElement('div');spacer.style.height='1400px';body.append(spacer);
  const before=[head.getBoundingClientRect().top,footer.getBoundingClientRect().top];
  body.scrollTop=500;
  check('view '+index+' scrolls only body',body.scrollTop===500&&document.querySelector('.main').scrollTop===0);
  check('view '+index+' header/footer stationary',head.getBoundingClientRect().top===before[0]&&footer.getBoundingClientRect().top===before[1]);
  check('view '+index+' footer inside frame',footer.getBoundingClientRect().bottom<=card.getBoundingClientRect().bottom);
  spacer.remove();body.scrollTop=0;
 }
 i=3;show();
 const main=document.querySelector('.main'),fullWidth=main.getBoundingClientRect().width;
 document.getElementById('right-panel-toggle').click();
 check('dock shrinks center',main.getBoundingClientRect().width<fullWidth);
 check('dock does not overlap',Math.abs(main.getBoundingClientRect().right-inspector.getBoundingClientRect().left)<1);
 check('dock is nonmodal',!document.querySelector('.shell').inert&&document.getElementById('side-panel-backdrop').hidden);
 check('right and left visual parity',getComputedStyle(inspector).backgroundColor===getComputedStyle(document.querySelector('.rail')).backgroundColor);
 document.getElementById('sidebar-toggle').click();
 check('independent left toggle',!inspector.hidden&&document.querySelector('.win').classList.contains('sidebar-collapsed'));
 document.getElementById('sidebar-toggle').click();
 selectedOutlineTask=1;renderOutlineSelection('b');
 check('selection updates open dock',inspector.querySelector('[data-detail-field="ownership"]').textContent==='tests/auth/');
 document.getElementById('right-panel-toggle').click();
 check('toggle restores center width',inspector.hidden&&main.getBoundingClientRect().width===fullWidth);
 selectRail('agents');await wait();
 const quota=document.querySelector('.usage-button[data-usage="claude"]');quota.click();await wait();
 const windows=[...document.querySelectorAll('.usage-item')],rings=[...quota.querySelectorAll('.usage-value')];
 check('quota windows have distinct colors',getComputedStyle(rings[0]).stroke!==getComputedStyle(rings[1]).stroke);
 check('quota ring and popup colors match',windows.every((item,index)=>getComputedStyle(item.querySelector('.usage-meter>span')).backgroundColor===getComputedStyle(rings[index]).stroke));
 check('quota legend matches meter',windows.every(item=>getComputedStyle(item.querySelector('.usage-line>span'),'::before').backgroundColor===getComputedStyle(item.querySelector('.usage-meter>span')).backgroundColor));
 document.querySelector('.usage-close').click();
 const icon=document.querySelector('.status-icon'),toolbar=document.querySelector('#right-panel-toggle svg');
 const sizes=[icon.getBoundingClientRect().width,getComputedStyle(icon).borderTopWidth,toolbar.getBoundingClientRect().width];
 const fill=document.documentElement.dataset.previewFill;document.documentElement.dataset.previewFill='true';
 check('full-window preserves icon and border sizes',JSON.stringify(sizes)===JSON.stringify([icon.getBoundingClientRect().width,getComputedStyle(icon).borderTopWidth,toolbar.getBoundingClientRect().width]));
 document.documentElement.dataset.previewFill=fill;
 for(const id of ['workspace-overview','team-brief-dialog','coordinator-chat-panel','settings-panel']){
  openSidePanel(id);check(id+' no focus scrolling',document.querySelector('.win').scrollLeft===0);
  closeSidePanel(id);openSidePanel(id);await wait();
  check(id+' rapid reopen survives timer',!document.getElementById(id).hidden&&document.getElementById(id).classList.contains('is-open'));
  closeSidePanel(id);await wait();
  check(id+' releases background',document.getElementById(id).hidden&&!document.querySelector('.shell').inert);
 }
 const themeSelect=document.getElementById('mock-theme');
 themeSelect.value=initialTheme==='dark'?'light':'dark';themeSelect.dispatchEvent(new Event('change'));
 check('theme changes locally',document.documentElement.dataset.theme!==initialTheme);
 themeSelect.value=initialTheme;themeSelect.dispatchEvent(new Event('change'));
 for(const index of [2,3,4,5,6,7]){
  i=index;show();renderTeamBriefing();
  const cards=[...document.querySelectorAll('.briefing-session')];
  check('view '+index+' factual briefing fields',cards.length===3&&cards.every(card=>['세션:','모델:','시도','프로세스:','메모리 보고','다음 단계:'].every(text=>card.textContent.includes(text))));
 }
 i=0;show();const priorCoordinator=JSON.stringify(workspaceCoordinator);
 openAssignmentPicker('coordinator','agent');check('Claude Code display name',[...document.querySelectorAll('#assignment-options button')].some(button=>button.dataset.value==='claude'&&button.textContent.includes('Claude Code')));
 closeSelectionMenu(true);check('dismiss preserves assignment',JSON.stringify(workspaceCoordinator)===priorCoordinator);
 openAssignmentPicker('coordinator','agent');document.querySelector('#assignment-options [data-value="claude"]').click();
 check('coordinator selection immediate',selectionMenu.hidden&&workspaceCoordinator.agent==='claude'&&roleAssignments[0].agent==='claude');
 openAssignmentPicker('coordinator','model');check('agent-specific model catalog',!!document.querySelector('#assignment-options [data-value="sonnet"]'));
 document.querySelector('#assignment-options [data-value="sonnet"]').click();check('model selection immediate',workspaceCoordinator.model==='sonnet');
 i=1;show();openAssignmentPicker('role-1','fallback');
 check('fallback excludes primary',!document.querySelector('#assignment-options [data-value="claude"]'));
 document.querySelector('#assignment-options [data-value="aipro"]').click();check('fallback applies and resets model',roleAssignments[1].fallback==='aipro'&&roleAssignments[1].fallbackModel==='');
 const fallbackButton=document.querySelector('[data-assignment-target="role-1"][data-assignment-field="fallback"]');
 check('fallback arrow separates pairs',fallbackButton.previousElementSibling.classList.contains('assignment-fallback-arrow'));
 openAssignmentPicker('role-1','fallbackModel');check('unsupported catalog offers default/manual',document.querySelectorAll('#assignment-options button').length===2);
 document.querySelector('#assignment-options [data-value="__custom"]').click();
 const manual=document.getElementById('assignment-manual-model');check('manual requires value',!document.getElementById('assignment-manual-form').checkValidity());manual.value='team-fallback-example';document.getElementById('assignment-manual-form').requestSubmit();
 check('fallback model saved',roleAssignments[1].fallbackModel==='team-fallback-example'&&selectionMenu.hidden);
 openAssignmentPicker('role-1','fallback');document.querySelector('#assignment-options [data-value=""]').click();check('no fallback disables its model',document.querySelector('[data-assignment-target="role-1"][data-assignment-field="fallbackModel"]').disabled);
 openSidePanel('settings-panel');await wait();
 const themeButton=nativeSelectionButtons.get(document.getElementById('mock-theme'));themeButton.click();
 check('settings uses same anchored menu',!selectionMenu.hidden&&selectionAnchor===themeButton&&!selectionMenu.hasAttribute('aria-modal'));
 const bounds=selectionMenu.getBoundingClientRect(),anchor=themeButton.getBoundingClientRect();check('choice menu anchored',Math.abs(bounds.left-anchor.left)<2&&Math.abs(bounds.top-anchor.bottom-6)<2);
 closeSelectionMenu();closeSidePanel('settings-panel');await wait();
 i=7;show();showGoalResult('rotation');document.getElementById('goal-result-dialog').close();openWorkspace();await wait();
 const resultButton=document.querySelector('.goal-row-actions button');resultButton.click();
 check('list uses shared result popup',document.getElementById('goal-result-dialog').open);
 document.getElementById('goal-result-dialog').close();
 check('result leaves list in place',document.getElementById('workspace-overview').classList.contains('is-open'));
 const trash=document.querySelector('.goal-trash'),row=trash.closest('.goal-list-row');
 check('trash separated from result/folder',row.firstElementChild===trash&&!row.querySelector('.goal-row-actions').contains(trash));
 trash.click();const deletion=document.getElementById('delete-goal-dialog');
 check('delete confirmation and safe focus',deletion.open&&document.activeElement.value==='cancel');
 await closeDialog(deletion,'cancel');check('cancel preserves goal',workspaceGoals.length===1);
 trash.click();await closeDialog(deletion,'delete');check('mock last-goal removal works',workspaceGoals.length===0);
 document.getElementById('overview-new-goal').click();
 check('new goal after empty',selectedGoalId==='draft');
 document.getElementById('goal-entry').value='A replacement Goal';
 document.querySelector('section.card[data-show="j e"] .btn.p').click();
 check('save new goal returns to roles',workspaceGoals.length===1&&i===1);
 const arrowPositions=[];
 for(let index=0;index<views.length;index++){
  if(document.getElementById('goal-result-dialog').open)document.getElementById('goal-result-dialog').close();
  i=index;show();
  arrowPositions.push(['prev','next'].map(id=>{const r=document.getElementById(id).getBoundingClientRect();return[r.x,r.y]}));
 }
 check('preview arrows stable',arrowPositions.every(pos=>JSON.stringify(pos)===JSON.stringify(arrowPositions[0])));
 check('pager below image',getComputedStyle(document.querySelector('.pager')).position==='static');
 check('ellipsis labels retain title', [...document.querySelectorAll('.tab-label')].every(el=>getComputedStyle(el).textOverflow==='ellipsis'&&el.title.endsWith(el.textContent)));
 check('phase groups share distinct backgrounds',new Set([...document.querySelectorAll('.step-group')].map(group=>getComputedStyle(group).backgroundColor)).size===3&&[...document.querySelectorAll('.step-group')].map(group=>group.querySelectorAll('.tab').length).join('/')==='3/2/1');
 check('phase group icons retain names',[...document.querySelectorAll('.tabs-group-name')].map(el=>el.getAttribute('aria-label')).join('/')==='준비/실행/마무리');
 check('enabled buttons use pointer',[...document.querySelectorAll('button:not(:disabled)')].every(button=>getComputedStyle(button).cursor==='pointer'));
 check('agent trash uses shared SVG',[...document.querySelectorAll('.agent-remove')].every(button=>button.querySelector('svg path').getAttribute('d')==='M3 6h18M9 6V4h6v2M5 6l1 14h12l1-14M10 10v6M14 10v6'));
 check('agent brands replace card initials',document.querySelectorAll('.agent .status-icon .agent-brand-logo').length===2);
 check('agent trash balances tools',[...document.querySelectorAll('.agent-remove svg')].every(svg=>getComputedStyle(svg).width==='13px'));
 check('inspector title is not a tab',getComputedStyle(document.querySelector('#task-inspector-title')).borderTopWidth==='0px'&&getComputedStyle(document.querySelector('#task-inspector-title')).backgroundColor==='rgba(0, 0, 0, 0)');
 check('evidence summary has no KPI tiles',[...document.querySelectorAll('.result-stats>div')].every(item=>getComputedStyle(item).backgroundColor==='rgba(0, 0, 0, 0)'&&getComputedStyle(item.querySelector('strong')).fontSize==='16px'));
 check('no script errors',!document.documentElement.dataset.designError);
 document.documentElement.dataset.interactionResult=JSON.stringify({theme:initialTheme,width:innerWidth,checks});
})().catch(error=>document.documentElement.dataset.designError=error.message);
