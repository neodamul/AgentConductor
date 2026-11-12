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
 i=0;show();const originalProfile=JSON.stringify(profileFixtures.claude);openProfileEditor('claude');
 check('executable is visible editable',!profileField('executable').readOnly&&profileField('executable').value==='claude');profileField('executable').value='example-cli';
 await closeDialog(profileDialog,'cancel');check('profile cancel preserves values',JSON.stringify(profileFixtures.claude)===originalProfile);
 openProfileEditor('claude');profileField('executable').value='example-cli';profileField('args').value='not JSON';document.getElementById('profile-edit-form').requestSubmit();check('profile rejects invalid args',profileDialog.open&&!!profileField('edit-error').textContent);
 profileField('args').value='[]';let profileClosed=new Promise(resolve=>profileDialog.addEventListener('close',resolve,{once:true}));document.getElementById('profile-edit-form').requestSubmit();await profileClosed;check('profile applies and closes',!profileDialog.open&&profileFixtures.claude.executable==='example-cli');
 openProfileEditor('claude');const providerButton=nativeSelectionButtons.get(profileField('provider'));providerButton.click();check('choice menu belongs to active modal',!selectionMenu.hidden&&selectionMenu.closest('dialog')===profileDialog);closeSelectionMenu();await closeDialog(profileDialog,'cancel');
 i=3;show();openClosePreview();const closing=document.getElementById('close-run-dialog');check('close default keeps background',closing.open&&document.activeElement.value==='keep');await closeDialog(closing,'cancel');check('close cancel recorded safely',document.documentElement.dataset.lastCloseChoice==='cancel');
 openClosePreview();await closeDialog(closing,'keep');check('background choice does not stop preview',document.documentElement.dataset.lastCloseChoice==='keep');openClosePreview();await closeDialog(closing,'stop');check('stop choice only simulates close',document.documentElement.dataset.lastCloseChoice==='stop');
 const priorCoordinator=JSON.stringify(workspaceCoordinator);
 openAssignmentPicker('coordinator','agent');check('Claude Code display name',[...document.querySelectorAll('#assignment-options button')].some(button=>button.dataset.value==='claude'&&button.textContent.includes('Claude Code')));
 closeSelectionMenu(true);check('dismiss preserves assignment',JSON.stringify(workspaceCoordinator)===priorCoordinator);
 openAssignmentPicker('coordinator','agent');document.querySelector('#assignment-options [data-value="claude"]').click();
 check('coordinator selection immediate',selectionMenu.hidden&&workspaceCoordinator.agent==='claude'&&roleAssignments[0].agent==='claude');
 openAssignmentPicker('coordinator','model');check('agent-specific model catalog',!!document.querySelector('#assignment-options [data-value="sonnet"]'));
 document.querySelector('#assignment-options [data-value="sonnet"]').click();check('model selection immediate',workspaceCoordinator.model==='sonnet');
 i=1;show();openAssignmentPicker('role-1','fallback');
 check('fallback excludes primary',!document.querySelector('#assignment-options [data-value="claude"]'));
 document.querySelector('#assignment-options [data-value="custom-cli"]').click();check('fallback applies and resets model',roleAssignments[1].fallback==='custom-cli'&&roleAssignments[1].fallbackModel==='');
 const fallbackButton=document.querySelector('[data-assignment-target="role-1"][data-assignment-field="fallback"]');
 check('fallback arrow separates pairs',fallbackButton.previousElementSibling.classList.contains('assignment-fallback-arrow'));
 openAssignmentPicker('role-1','fallbackModel');check('unsupported catalog offers default/manual',document.querySelectorAll('#assignment-options button').length===2);
 document.querySelector('#assignment-options [data-value="__custom"]').click();
 const manual=document.getElementById('assignment-manual-model');check('manual requires value',!document.getElementById('assignment-manual-form').checkValidity());manual.value='team-fallback-example';document.getElementById('assignment-manual-form').requestSubmit();
 check('fallback model saved',roleAssignments[1].fallbackModel==='team-fallback-example'&&selectionMenu.hidden);
 openAssignmentPicker('role-1','fallback');document.querySelector('#assignment-options [data-value=""]').click();check('no fallback disables its model',document.querySelector('[data-assignment-target="role-1"][data-assignment-field="fallbackModel"]').disabled);
 {const saved=roleAssignments.map(item=>({...item})),coordinator={...workspaceCoordinator};document.querySelector('[data-role-defaults]').click();check('apply agent role defaults assigns every role',roleAssignments.every(item=>item.agent&&item.fallback&&item.agent!==item.fallback&&!item.model)&&roleAssignments[1].agent==='codex'&&document.querySelector('[data-assignment-target="role-1"][data-assignment-field="agent"] span').textContent==='Codex');saved.forEach((item,k)=>Object.assign(roleAssignments[k],item));Object.assign(workspaceCoordinator,coordinator);refreshAssignmentChoices();}
 openSidePanel('settings-panel');await wait();
 const themeButton=nativeSelectionButtons.get(document.getElementById('mock-theme'));themeButton.click();
 check('settings uses same anchored menu',!selectionMenu.hidden&&selectionAnchor===themeButton&&!selectionMenu.hasAttribute('aria-modal'));
 const bounds=selectionMenu.getBoundingClientRect(),anchor=themeButton.getBoundingClientRect();check('choice menu anchored',Math.abs(bounds.left-anchor.left)<2&&Math.abs(bounds.top-anchor.bottom-6)<2);
 closeSelectionMenu();closeSidePanel('settings-panel');await wait();
 i=7;show();showGoalResult('rotation');document.getElementById('goal-result-dialog').close();openWorkspace();await wait();
 const resultButton=document.querySelector('.goal-row-actions button');resultButton.click();
 check('list uses shared result popup',document.getElementById('goal-result-dialog').open);
 {const sections=[...document.querySelectorAll('#result-detail details.result-section')];check('result sections start collapsed',sections.length===3&&sections.every(d=>!d.open));sections[0].querySelector('summary').click();check('a result section expands on click',sections[0].open&&sections[0].querySelector('li').offsetHeight>0);sections[0].querySelector('summary').click();check('a result section collapses again',!sections[0].open);}
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
 check('enabled buttons use pointer',[...document.querySelectorAll('button:not(:disabled):not(.workspace-home):not(.is-busy)')].every(button=>getComputedStyle(button).cursor==='pointer'));
 check('agent trash uses shared SVG',[...document.querySelectorAll('.agent-remove')].every(button=>button.querySelector('svg path').getAttribute('d')==='M3 6h18M9 6V4h6v2M5 6l1 14h12l1-14M10 10v6M14 10v6'));
 check('agent brands replace card initials',document.querySelectorAll('.agent .status-icon .agent-brand-logo').length===2);
 check('agent trash balances tools',[...document.querySelectorAll('.agent-remove svg')].every(svg=>getComputedStyle(svg).width==='13px'));
 check('inspector title is not a tab',getComputedStyle(document.querySelector('#task-inspector-title')).borderTopWidth==='0px'&&getComputedStyle(document.querySelector('#task-inspector-title')).backgroundColor==='rgba(0, 0, 0, 0)');
 check('evidence summary has no KPI tiles',[...document.querySelectorAll('.result-stats>div')].every(item=>getComputedStyle(item).backgroundColor==='rgba(0, 0, 0, 0)'&&getComputedStyle(item.querySelector('strong')).fontSize==='16px'));
 check('goal counts stay synchronized',[...document.querySelectorAll('[data-goal-count]')].every(badge=>badge.textContent===String(workspaceGoals.length)));
 check('goal list looks like a button',getComputedStyle(document.getElementById('goal-picker')).borderTopWidth==='1px');
 check('briefing icon beside goal list',document.getElementById('goal-picker').nextElementSibling.id==='team-brief-button'&&!!document.querySelector('#team-brief-button svg'));
 check('tab captions left aligned',[...document.querySelectorAll('.tabs .tab-label,.ttabs>button,.rail-tabs button')].every(tab=>getComputedStyle(tab).textAlign==='left'));
 check('card actions have leading icons',[...document.querySelectorAll('.main>.card button')].filter(button=>!button.closest('.ttabs')&&!button.querySelector('svg')).every(button=>!!button.dataset.actionIcon));
 i=3;show();check('task rows show progress tags',[...document.querySelectorAll('.outline-state-badge')].map(tag=>tag.textContent).join('/')==='실행 중/대기/완료');
 check('tasks look selectable',document.querySelectorAll('.outline-task .outline-open-icon').length===3);
 check('task avatars stay compact beside tags',[...document.querySelectorAll('.outline-agent')].every(icon=>getComputedStyle(icon).width==='28px')&&document.querySelectorAll('.outline-agent .agent-brand-logo').length===2);
 i=5;show();const blockedTag=document.querySelectorAll('.outline-state-badge')[2],tone=document.createElement('span');tone.style.cssText='color:var(--err);background:var(--err-bg)';document.body.append(tone);check('blocked task keeps red semantic tag',blockedTag.textContent==='막힘'&&getComputedStyle(blockedTag).color===getComputedStyle(tone).color&&getComputedStyle(blockedTag).backgroundColor===getComputedStyle(tone).backgroundColor);tone.remove();
 check('Goal list buttons share icon',[...document.querySelectorAll('.goal-list-button')].length===1&&document.querySelectorAll('.goal-list-button .goal-list-icon').length===1);
 {const view=i;i=0;show();const head=document.querySelector('.head'),section=document.querySelector('.workspace-goal-section');check('Workspace step shows Goal list and team briefing in the top bar',getComputedStyle(document.getElementById('goal-picker')).display!=='none'&&getComputedStyle(document.getElementById('team-brief-button')).display!=='none'&&head.contains(document.getElementById('goal-picker')));check('Workspace Goal section keeps only New Goal',section.querySelectorAll('.goal-section-actions button').length===1&&section.querySelector('#workspace-new-goal')&&!section.querySelector('.goal-list-button,.team-brief-button'));i=view;show();}
 {const saved=mcpServers.map(server=>({...server,secrets:server.secrets.map(pair=>[...pair]),tools:[...server.tools]})),delay=mcpDiscoverDelay;mcpDiscoverDelay=0;document.getElementById('mcp-open').click();
  check('MCP list opens from Settings',mcpDialog.open&&document.querySelectorAll('#mcp-servers .mcp-server').length===2&&mcpField('edit-view').hidden&&!mcpField('list-view').hidden);
  check('MCP settings has no manual binding form or duplicate tool list',!document.getElementById('mcp-jira')&&!document.getElementById('mcp-confluence')&&!document.getElementById('mcp-tools')&&mcpServers[0].tools.length>0);
  mcpField('add').click();check('MCP add form keeps raw JSON in Advanced',!mcpField('edit-view').hidden&&mcpField('list-view').hidden&&!mcpField('edit-view').querySelector('textarea'));
  mcpField('kind').value='local';mcpField('kind').dispatchEvent(new Event('change'));check('local MCP form asks for command and environment',mcpField('url-label').hidden&&!mcpField('command-label').hidden&&mcpField('secret-title').textContent==='환경 변수');
  mcpField('edit-cancel').click();mcpField('close').click();check('MCP dialog closes',!mcpDialog.open);mcpServers.splice(0,mcpServers.length,...saved);mcpDiscoverDelay=delay;}
 {const added=!workspaceGoals.some(item=>item.id==='rotation');if(added)workspaceGoals.push({id:'rotation',name:'리프레시 토큰 회전을 추가한다',slug:'refresh-token-rotation',state:'running',step:3,position:'Phase 1/2',summary:'',commit:'3f9a1c2'});const goal=workspaceGoals.find(item=>item.id==='rotation'),saved={state:goal.state,step:goal.step,position:goal.position,summary:goal.summary},view=i;goal.nextPhase={number:2,total:2};showGoalResult('rotation');
  const sections=[...document.querySelectorAll('#result-detail .result-section')];check('result sections show counts and explicit expand toggles',sections.length===3&&sections.every(section=>{const summary=section.querySelector('summary'),count=summary.querySelector('.result-section-count'),toggle=summary.querySelector('.result-section-toggle');return !section.open&&summary.getAttribute('aria-expanded')==='false'&&count.textContent.trim().endsWith(String(section.querySelectorAll('li').length))&&toggle.textContent==='펼치기';}));
  const first=sections[0],summary=first.querySelector('summary');summary.click();await wait();check('result heading expands and updates accessible state',first.open&&summary.getAttribute('aria-expanded')==='true'&&summary.querySelector('.result-section-toggle').textContent==='접기');summary.click();await wait();check('result heading collapses and updates accessible state',!first.open&&summary.getAttribute('aria-expanded')==='false'&&summary.querySelector('.result-section-toggle').textContent==='펼치기');
  const action=document.getElementById('result-to-workspace');check('a Phase with Phases left offers the next Phase plan',document.getElementById('goal-result-eyebrow').textContent==='Phase 1/2 완료'&&action.textContent.trim()==='다음 Phase 계획 (2/2)');
  action.click();check('the next Phase plan opens step 3',!document.getElementById('goal-result-dialog').open&&i===2&&!document.getElementById('workspace-overview').classList.contains('is-open'));
  delete goal.nextPhase;showGoalResult('rotation');check('the last Phase offers the Goal list',document.getElementById('goal-result-eyebrow').textContent==='목표 완료'&&action.textContent.trim()==='목표 목록으로');
  document.getElementById('goal-result-dialog').close();Object.assign(goal,saved);if(added)workspaceGoals.splice(workspaceGoals.indexOf(goal),1);i=view;show();}
 {selectRail('agents');await wait();const coord=document.querySelector('.agent.coordinator'),role=coord.querySelector('.fact.role'),css=getComputedStyle(document.documentElement);
  check('the coordinator tag leads its card in the role color with a role band',coord.querySelector('.work-facts').firstElementChild===role&&getComputedStyle(role).color!==getComputedStyle(coord.querySelector('.fact.info')).color&&getComputedStyle(coord).boxShadow.includes('inset')&&document.querySelectorAll('.fact.role').length===1);
  const tags=[...document.querySelectorAll('.agent .fact')];check('every tag has a thin border and 4px corners',tags.length>8&&tags.every(t=>{const c=getComputedStyle(t);return c.borderTopWidth==='1px'&&c.borderTopStyle==='solid'&&c.borderTopLeftRadius==='4px';}));
  check('tags never cover the card tools',[...document.querySelectorAll('#rail-agents-panel .agent')].every(card=>{const facts=[...card.querySelectorAll('.facts')].pop().getBoundingClientRect(),tools=card.querySelector('.tools').getBoundingClientRect(),box=card.getBoundingClientRect();return facts.bottom<=tools.top+0.5&&tools.bottom<=box.bottom+0.5;}));
  check('work states use their state colors, info tags stay neutral',getComputedStyle(document.querySelector('[data-state-tag="failed"]')).color!==getComputedStyle(document.querySelector('[data-state-tag="done"]')).color&&getComputedStyle(document.querySelector('.agent .fact.info')).backgroundColor!==getComputedStyle(document.querySelector('[data-state-tag="running"]')).backgroundColor);
  const models=document.querySelector('[data-sessions="claude"]').closest('.agent').querySelector('.fact.models'),list=models.querySelector('.model-list');
  check('the model list is hidden until the models tag is hovered or focused',getComputedStyle(list).display==='none');
  models.focus();models.dispatchEvent(new FocusEvent('focusin',{bubbles:true}));check('focusing the models tag lists its models, capped with a remainder line',getComputedStyle(list).display==='block'&&list.querySelectorAll('span').length===9&&list.querySelector('.more').textContent==='+4개 더'&&models.getAttribute('aria-describedby')===list.id);
  models.blur();models.dispatchEvent(new FocusEvent('focusout',{bubbles:true}));check('leaving the models tag closes the list',getComputedStyle(list).display==='none');}
 {const saved=JSON.parse(JSON.stringify(agentSessions));selectRail('agents');await wait();
  const button=document.querySelector('[data-sessions="claude"]');check('agent cards count their sessions',button.querySelector('.session-count').textContent==='3'&&[...button.parentElement.querySelectorAll('[data-state-tag]')].map(t=>t.textContent).join(',')==='실행 중1,실패1,완료1'&&document.querySelector('[data-sessions="custom-cli"] .session-count').textContent==='0');
  button.click();check('sessions open per agent with a state summary',sessionsDialog.open&&sessionsField('title').textContent==='Claude Code · 세션'&&document.querySelectorAll('#sessions-list .session-row').length===3&&sessionsField('summary').textContent.includes('실행 중 1'));
  const rows=[...document.querySelectorAll('#sessions-list .session-row')];check('each row shows state, task, Goal and times',rows[0].querySelector('.session-state.running')&&rows[0].textContent.includes('T1 토큰 회전 구현')&&rows[0].textContent.includes('시작 21:02')&&rows[2].querySelector('.session-state.stale'));
  check('running sessions stop, finished ones clean up',rows[0].querySelector('button').textContent==='중지'&&rows[1].querySelector('button').textContent==='정리'&&sessionsField('clean-finished').textContent==='끝난 세션 정리 (2)');
  rows[0].querySelector('button').click();check('stopping keeps the session as stopped',document.querySelector('#sessions-list .session-row .session-state').textContent==='중지됨'&&sessionsField('clean-finished').textContent==='끝난 세션 정리 (3)'&&!button.parentElement.querySelector('[data-state-tag="running"]'));
  sessionsField('clean-finished').click();check('cleanup asks for confirmation first',!sessionsField('confirm').hidden&&document.querySelectorAll('#sessions-list .session-row').length===3);
  sessionsField('confirm-cancel').click();check('cancel keeps every session',sessionsField('confirm').hidden&&document.querySelectorAll('#sessions-list .session-row').length===3);
  sessionsField('clean-finished').click();sessionsField('confirm-ok').click();check('confirmed cleanup removes finished sessions and updates the card',document.querySelectorAll('#sessions-list .session-row').length===0&&button.querySelector('.session-count').textContent==='0'&&sessionsField('clean-finished').disabled);
  sessionsField('close').click();document.querySelector('[data-sessions="custom-cli"]').click();check('an agent without sessions shows an empty state',document.querySelector('#sessions-list .panel-note')?.textContent.includes('세션이 없습니다'));sessionsField('close').click();
  Object.keys(agentSessions).forEach(k=>agentSessions[k]=saved[k]);renderSessionButtons();check('sessions close',!sessionsDialog.open);}
 {const view=i;i=3;show();const toggle=document.getElementById('goal-title-toggle'),text=document.getElementById('goal-title-text');
  check('the task step shows the Goal folded with an Expand toggle',getComputedStyle(toggle).display!=='none'&&toggle.textContent==='▸ 펼치기'&&text.hidden&&getComputedStyle(document.getElementById('saved-goal-title')).whiteSpace==='nowrap');
  toggle.click();check('Expand shows the whole Goal',!text.hidden&&text.textContent===(selectedGoal().description||selectedGoal().name)&&text.textContent.length>0&&toggle.textContent==='▾ 접기'&&toggle.getAttribute('aria-expanded')==='true');
  toggle.click();check('Collapse folds it again',text.hidden&&toggle.textContent==='▸ 펼치기');
  i=0;show();check('the Workspace step has no Goal toggle',getComputedStyle(toggle).display==='none'&&getComputedStyle(text).display==='none');i=view;show();}
 {const view=i;i=7;show();showGoalResult('rotation');const files=document.querySelector('#result-detail .result-section');files.open=true;const row=files.querySelector('.file-diff-toggle'),pre=files.querySelector('.file-diff');
  check('changed file rows are buttons with a folded diff',!!row&&row.getAttribute('aria-expanded')==='false'&&pre.hidden&&row.textContent.length>0);
  row.click();check('a click unfolds that file\'s diff with added and removed lines',!pre.hidden&&pre.querySelector('.diff-line.hunk')&&pre.querySelector('.diff-line.add')&&row.getAttribute('aria-expanded')==='true'&&files.querySelectorAll('.file-diff:not([hidden])').length===1);
  row.click();check('a second click folds it again',pre.hidden&&row.getAttribute('aria-expanded')==='false');
  document.getElementById('goal-result-dialog').close();i=view;show();}
 {const view=i;i=2;show();const card=[...document.querySelectorAll('.main>.card')].find(el=>getComputedStyle(el).display!=='none');const button=[...card.querySelectorAll('.actions .btn')].find(b=>!b.disabled);
  const before=getComputedStyle(button).backgroundColor;button.click();const bar=getComputedStyle(button,'::after');
  check('a pressed step action shows a moving bar on its bottom edge',button.classList.contains('is-busy')&&button.getAttribute('aria-busy')==='true'&&bar.animationName==='ac-busy'&&bar.height==='3px'&&bar.bottom==='0px'&&bar.position==='absolute'&&getComputedStyle(button).cursor==='progress');
  check('the busy action keeps its own colors',getComputedStyle(button).backgroundColor===before&&!button.disabled);
  const cancel=document.getElementById('profile-edit-cancel');check('every work action can show the bar, openers and Cancel cannot',[...document.querySelectorAll('.btn')].filter(startsWork).length>=25&&!startsWork(cancel)&&!startsWork(document.getElementById('mcp-open')));
  await new Promise(resolve=>setTimeout(resolve,1300));check('the bar ends when the work finishes',!button.classList.contains('is-busy')&&!button.hasAttribute('aria-busy'));i=view;show();}
 check('duplicate menu commands are absent (#865–867)',!document.getElementById('submenu-agents')&&!document.getElementById('submenu-view')&&!document.getElementById('menu-workspace-settings')&&!!document.getElementById('profile-add')&&!!document.getElementById('sidebar-toggle')&&!!document.getElementById('right-panel-toggle'));

 {const previous=window.open;let opened;try{window.open=(url,target,features)=>{opened={url,target,features};return null;};document.getElementById('menu-report-bug').click();check('Help opens a standard GitHub bug draft in the browser',opened?.url.startsWith('https://github.com/neodamul/AgentConductor/issues/new?body=')&&decodeURIComponent(opened.url).includes('## Steps to reproduce')&&opened.target==='_blank'&&opened.features==='noopener');}finally{window.open=previous;}}
 {const hint=document.getElementById('mcp-import-format'),advanced=document.getElementById('mcp-advanced');
  check('the JSON import says it uses the OpenCode MCP format beside the import controls',!!hint&&advanced.contains(hint)&&hint.previousElementSibling.querySelector('#mcp-import')&&hint.textContent.includes('OpenCode MCP')&&document.getElementById('mcp-json').getAttribute('aria-describedby')==='mcp-import-format');}
 {const hint=document.getElementById('mcp-secret-hint');check('#887: the server form says API tokens must be registered as environment variables',!!hint&&document.getElementById('mcp-edit-view').contains(hint)&&hint.textContent.includes('API 토큰')&&hint.textContent.includes('{env:VARIABLE}')&&hint.previousElementSibling.id==='mcp-secrets');}
 // #890: approved states for the 0.2.5 fixes must be reachable, not only described.
 {i=2;show();enablePhaseHistory();document.querySelector('[data-history-task]').click();
 check('#880: selecting an earlier Phase opens its read-only task',!document.getElementById('history-detail').hidden&&document.getElementById('history-detail').textContent.includes('리프레시 토큰')&&!document.getElementById('history-detail').querySelector('input,textarea'));
 document.querySelector('[data-history-task]').click();check('#880: reselecting the same history keeps it open',!document.getElementById('history-detail').hidden);
 document.querySelector('[data-current-history-task]').click();check('#880: current Phase restores the editable plan',document.getElementById('history-phase').hidden);}
 {i=3;show();openRawLogs();const log=document.getElementById('preview-stdout');check('#879: raw logs retain multiline text and metadata',log.textContent.includes('리프레시 토큰\n이전 토큰 무효화')&&log.textContent.includes('session_id')&&getComputedStyle(log).whiteSpace==='pre-wrap');document.querySelector('.card[data-show="b"] .ttabs button').click();}
 {openMessengerPreview();messengerPreviewState('saving');check('#881: pending save keeps the form open and disabled',messengerPreview.open&&document.getElementById('messenger-preview-save').disabled);messengerPreviewState('error');check('#881: failed save retains draft and inline error',messengerPreview.open&&!document.getElementById('messenger-preview-save').disabled&&!document.getElementById('messenger-preview-error').hidden&&getComputedStyle(document.getElementById('messenger-preview-error')).position==='static');messengerPreview.close();}
 {const view=i;const kinds=new Map();for(const v of [0,3,6]){i=v;show();for(const t of document.querySelectorAll('.pill,.outline-state-badge,.plan-tag'))if(t.getClientRects().length)kinds.set(t.className.split(' ')[0]+':'+(t.dataset.state||t.className),t);}
  openSessions('claude');for(const t of document.querySelectorAll('#sessions-list .session-state'))kinds.set('session:'+t.className,t);
  const tags=[...kinds.values()];check('#892: every status tag has a thin border and 4px corners',tags.length>=6&&tags.every(t=>{const c=getComputedStyle(t);return c.borderTopWidth==='1px'&&c.borderTopStyle==='solid'&&c.borderTopLeftRadius==='4px';}));
  const run=document.querySelector('#sessions-list .session-state.running'),fact=document.querySelector('[data-state-tag="running"]');check('#892: running uses the same info color everywhere',!!run&&!!fact&&getComputedStyle(run).color===getComputedStyle(fact).color);
  sessionsField('close').click();i=view;show();}
 {const input=document.getElementById('goal-entry'),before=input.value;i=0;show();input.value='😀 [Jira:EMM-51214] [Confluence:2571110463]';input.dispatchEvent(new Event('input'));const tags=[...document.querySelectorAll('#reference-goal-tags mark')];check('#898: bracketed source tags keep their exact text and visible background',tags.length===2&&tags[0].textContent==='[Jira:EMM-51214]'&&getComputedStyle(input).backgroundColor==='rgba(0, 0, 0, 0)'&&getComputedStyle(tags[0]).backgroundColor!=='rgba(0, 0, 0, 0)');input.value=before;input.dispatchEvent(new Event('input'));}
 {i=0;show();const goalInput=document.getElementById('goal-entry'),goalBefore=goalInput.value;goalInput.value='[Jira:EMM-52599] [Confluence:53553565] [Jira:EMM-52611]';goalInput.dispatchEvent(new Event('input'));document.getElementById('workspace-preview-documents').click();const d=document.getElementById('source-review'),cards=[...d.querySelectorAll('.source-doc')];
  check('#903: preview opens with the guide and one card per reference',d.open&&document.getElementById('source-guide').open&&cards.length===3&&document.getElementById('source-retry').hidden);
  check('#903: the running card shows its step and live log',cards[1].classList.contains('running')&&cards[1].querySelector('.source-step.current').textContent==='원문 조회'&&cards[1].querySelector('.source-log').textContent.includes('제한 300초'));
  check('#903: the Goal status names the running document',document.getElementById('source-status').textContent.includes('confluence:53553565'));
  document.getElementById('source-guide').querySelector('summary').click();renderSourceReview('running');check('#916: manual collapse survives progress refresh',!document.getElementById('source-guide').open);document.getElementById('source-guide').querySelector('summary').click();
  openSourceReview('review');check('#916: manual expansion survives completion',document.getElementById('source-guide').open);const done=[...d.querySelectorAll('.source-doc')];
  check('#903: review cards carry summaries and the failed step',done[0].querySelector('.source-summary')&&done[2].querySelector('.source-step.failed')&&done[2].querySelector('.source-error')&&!document.getElementById('source-retry').hidden&&document.getElementById('source-apply').disabled);
  document.getElementById('source-close').click();check('#903: Close leaves the lookup running',!d.open);goalInput.value=goalBefore;goalInput.dispatchEvent(new Event('input'));}
 {i=0;show();const input=document.getElementById('goal-entry'),before=input.value;input.value='Implement [Jira:AC-1]';input.dispatchEvent(new Event('input'));document.getElementById('workspace-save-only').click();check('#905: plain Save stays on Workspace and guides source preview',i===0&&!document.getElementById('workspace-source-reminder').hidden);input.value='';input.dispatchEvent(new Event('input'));check('#906: an empty Goal has the error border',input.getAttribute('aria-invalid')==='true');input.value=before;input.dispatchEvent(new Event('input'));check('#906: correcting the Goal clears its invalid state',input.getAttribute('aria-invalid')==='false');document.getElementById('workspace-source-reminder').hidden=true;}
 {i=0;show();const input=document.getElementById('goal-entry'),before=input.value,set=v=>{input.value=v;input.dispatchEvent(new Event('input'));},status=document.getElementById('source-status');
  set('[Jira:A-1] [Jira:A-2] [Confluence:3] [Confluence:4]');check('#907: four references show the count and allow adding',status.textContent==='참조 4/5'&&!document.getElementById('source-jira').disabled&&!document.getElementById('workspace-preview-documents').disabled);
  set('[Jira:A-1] [Jira:A-2] [Confluence:3] [Confluence:4] [Jira:A-1] [Jira:A-5]');check('#907: five distinct references stop adding but still preview',status.textContent==='참조 5/5'&&document.getElementById('source-jira').disabled&&document.getElementById('source-confluence').disabled&&!document.getElementById('workspace-preview-documents').disabled);
  set('[Jira:A-1] [Jira:A-2] [Confluence:3] [Confluence:4] [Jira:A-5] [Jira:A-6]');check('#907: a sixth reference blocks preview with an inline error',status.classList.contains('over')&&status.textContent.includes('5개까지')&&document.getElementById('workspace-preview-documents').disabled&&getComputedStyle(status).color!==getComputedStyle(document.getElementById('goal-entry')).color);
  set(before);check('#907: removing references restores the controls',!document.getElementById('source-jira').disabled&&status.textContent==='');document.getElementById('workspace-preview-documents').disabled=false;}
 check('no script errors',!document.documentElement.dataset.designError);
 document.documentElement.dataset.interactionResult=JSON.stringify({theme:initialTheme,width:innerWidth,checks});
})().catch(error=>document.documentElement.dataset.designError=error.message);
