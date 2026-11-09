function renderTaskDetails(card){
 const panel=card.matches('.task-detail-content')?card:card.querySelector('.task-detail-content');if(!panel)return;
 const task=outlineTasks[selectedOutlineTask],view=views[i][0],state=(agentStates[view]||idleState)[selectedOutlineTask];
 panel.querySelector('[data-detail-field="ownership"]').textContent=task.path;
 panel.querySelector('[data-detail-field="session"]').textContent=task.id+' · 세션: mock-'+task.agent+'-'+task.id;
 panel.querySelector('[data-detail-field="instruction"]').textContent=task.instruction;
 panel.querySelector('[data-detail-field="dependency"]').textContent=task.dependency;
 panel.querySelector('[data-detail-field="test"]').textContent=task.test;
 panel.querySelector('[data-detail-field="restriction"]').textContent=state[0]==='error'?state[1]:'기록된 사용 제한이 없습니다.';
 panel.querySelector('[data-detail-field="tools"]').textContent=view==='g'?'승인 대기 1건 · npm install jsonwebtoken@9 · 네트워크 접근':'대기 중인 도구 요청이 없습니다.';
 panel.querySelector('[data-detail-field="tokens"]').textContent='보고되지 않음';
 panel.querySelector('[data-detail-field="duration"]').textContent=document.getElementById('outline-elapsed').textContent;
 panel.querySelector('[data-detail-field="completion"]').textContent=document.getElementById('outline-completed').textContent;
 const agent=panel.querySelector('[data-detail-agent]');agent.value=task.previewAgent||task.agent;
 function quota(){panel.querySelector('[data-detail-field="quota"]').textContent=agent.value==='claude'?'남은 사용량 64% / 100% · 시안 데이터':agent.value==='codex'?'사용량 조회 오류: quota query timed out':'이 CLI는 사용량 조회를 제공하지 않습니다.';}
 agent.onchange=quota;quota();panel.querySelector('[data-detail-quota]').onclick=quota;
 panel.querySelector('[data-detail-assign]').onclick=()=>{task.previewAgent=agent.value;panel.querySelector('[data-detail-notice]').textContent='시안: '+agent.value+'로 재배정을 선택했습니다. 실제 세션은 변경되지 않습니다.';};
 panel.querySelector('[data-detail-model-save]').onclick=()=>{panel.querySelector('[data-detail-notice]').textContent='시안: CLI 기본 모델을 선택했습니다. 실제 세션은 변경되지 않습니다.';};
 panel.querySelector('[data-detail-schedule]').onclick=()=>{const time=panel.querySelector('[data-detail-resume]');if(!time.value){time.reportValidity();return;}panel.querySelector('[data-detail-notice]').textContent='시안: '+time.value.replace('T',' ')+' 재개 예약을 선택했습니다. 실제 예약은 생성하지 않습니다.';};
 panel.querySelector('[data-detail-review]').onclick=()=>{i=7;show();};
 if(typeof syncSelectionButtons==='function')syncSelectionButtons();
}
document.querySelectorAll('.main>.card[data-show="b"],.main>.card[data-show="g"],.main>.card[data-show="h"]').forEach(card=>{
 const body=card.querySelector('.card-body'),tabs=card.querySelector('.ttabs'),activity=document.createElement('div'),details=document.createElement('div');activity.className='task-activity-content';details.className='task-detail-content';details.hidden=true;
 [...body.children].filter(child=>child!==tabs).forEach(child=>activity.append(child));tabs.after(activity,details);
 details.innerHTML=`<p class="panel-note">선택한 작업의 상세정보 · 시안 데이터</p>
 <section><h3>소유 경로와 작업</h3><code data-detail-field="ownership"></code><p class="detail-muted" data-detail-field="session"></p><p data-detail-field="instruction"></p><dl><dt>의존성</dt><dd data-detail-field="dependency"></dd><dt>검증 명령</dt><dd><code data-detail-field="test"></code></dd></dl></section>
 <section><h3>담당과 모델</h3><label>에이전트<div class="detail-inline"><select data-detail-agent><option>claude</option><option>codex</option><option>custom-cli</option></select><button class="btn q" data-detail-assign>재배정</button></div></label><label>세션 모델<div class="detail-inline"><select><option>CLI 기본값</option></select><button class="btn q" data-detail-model-save>적용</button></div></label><p class="detail-muted">실제 앱에서는 해당 에이전트가 제공하는 모델 목록을 사용합니다.</p></section>
 <section><h3>가용성과 재개 예약</h3><p data-detail-field="restriction"></p><label>재개 시각 · 로컬 시간<div class="detail-inline"><input type="datetime-local" data-detail-resume required><button class="btn q" data-detail-schedule>예약</button></div></label></section>
 <section><h3>검토 후보</h3><p class="detail-muted">검증을 마친 통합 후보의 diff와 테스트를 확인한 뒤 승인합니다.</p><div class="detail-inline"><button class="btn q" data-detail-review>검토·병합로 이동</button><button class="btn q" disabled>승인</button><button class="btn q" disabled>거절</button></div><p class="detail-muted">현재 작업 단계에서는 머지할 수 없습니다.</p></section>
 <section><h3>도구 요청</h3><p data-detail-field="tools"></p></section>
 <section><h3>계정 사용량 <button class="detail-refresh" data-detail-quota aria-label="상세 사용량 새로고침">↻</button></h3><p data-detail-field="quota"></p><p class="detail-muted">계정이 공유하는 한도이며 잔여 현금이 아닙니다.</p></section>
 <section><h3>세션 지표</h3><dl><dt>보고된 토큰</dt><dd data-detail-field="tokens"></dd><dt>작업 경과</dt><dd data-detail-field="duration"></dd><dt>완료 작업</dt><dd data-detail-field="completion"></dd><dt>비용</dt><dd>보고되지 않음</dd></dl><p class="detail-muted">구독 비용을 토큰 수로 추정하지 않습니다.</p></section><p class="detail-muted" data-detail-notice role="status"></p>`;
 for(const label of ['활동','상세']){const old=[...tabs.children].find(el=>el.textContent===label),button=document.createElement('button');button.type='button';button.textContent=label;button.className=old.className;button.setAttribute('aria-pressed',String(label==='활동'));old.replaceWith(button);button.onclick=()=>{const open=label==='상세';activity.hidden=open;details.hidden=!open;tabs.querySelectorAll('.on').forEach(tab=>tab.classList.remove('on'));button.classList.add('on');tabs.querySelectorAll('button').forEach(tab=>tab.setAttribute('aria-pressed',String(tab===button)));if(open)renderTaskDetails(card);body.scrollTop=0;};}
});
const inspectorDetails=document.querySelector('.card[data-show="b"] .task-detail-content');
document.getElementById('task-inspector-body').append(inspectorDetails);inspectorDetails.hidden=false;
document.querySelectorAll('.card .task-detail-content').forEach(detail=>detail.remove());
function setTaskInspector(open){
 const panel=document.getElementById('task-inspector-panel');panel.hidden=!open;document.querySelector('.win').classList.toggle('inspector-open',open);const toggle=document.getElementById('right-panel-toggle');toggle.setAttribute('aria-expanded',String(open));toggle.setAttribute('aria-label',open?'오른쪽 작업 상세 패널 접기':'오른쪽 작업 상세 패널 펼치기');
}
function openTaskInspector(){
 const view=views[i][0],ready=!['j','e','i'].includes(view);
 inspectorDetails.hidden=!ready;
 document.getElementById('task-inspector-context').textContent=ready?outlineTasks[selectedOutlineTask].agent+' · '+outlineTasks[selectedOutlineTask].id+' · '+outlineTasks[selectedOutlineTask].title:'계획을 만든 뒤 작업을 선택하면 상세정보가 표시됩니다.';
 if(ready)renderTaskDetails(inspectorDetails);
 setTaskInspector(true);
}
document.getElementById('right-panel-toggle').onclick=()=>{if(!document.getElementById('task-inspector-panel').hidden)setTaskInspector(false);else openTaskInspector();};
document.querySelectorAll('.card .ttabs button').forEach(button=>{
 if(button.textContent==='상세')button.remove();
 else if(button.textContent==='활동')button.onclick=null;
});
const appMenu=document.getElementById('app-menu'),menuButton=document.getElementById('app-menu-button');
function closeAppMenu(){appMenu.hidden=true;menuButton.setAttribute('aria-expanded','false');}
menuButton.onclick=()=>{appMenu.style.left=menuButton.offsetLeft+'px';appMenu.style.top=(menuButton.offsetTop+menuButton.offsetHeight+4)+'px';appMenu.hidden=!appMenu.hidden;menuButton.setAttribute('aria-expanded',String(!appMenu.hidden));};
document.addEventListener('click',event=>{if(!appMenu.contains(event.target)&&!menuButton.contains(event.target))closeAppMenu();});
document.addEventListener('keydown',event=>{if(event.key==='Escape'&&!appMenu.hidden){closeAppMenu();menuButton.focus();}});
document.getElementById('sidebar-toggle').onclick=()=>{const collapsed=document.querySelector('.win').classList.toggle('sidebar-collapsed');const button=document.getElementById('sidebar-toggle');button.setAttribute('aria-expanded',String(!collapsed));button.title=collapsed?'왼쪽 패널 펼치기':'왼쪽 패널 접기';button.setAttribute('aria-label',button.title);document.getElementById('left-sidebar').inert=collapsed;};
document.getElementById('coordinator-chat-button').onclick=()=>openSidePanel('coordinator-chat-panel');
document.getElementById('settings-button').onclick=()=>openSidePanel('settings-panel');
document.getElementById('menu-workspace').onclick=()=>{closeAppMenu();openWorkspaceHome();};
document.getElementById('menu-goals').onclick=()=>{closeAppMenu();openWorkspace();};
document.getElementById('menu-settings').onclick=()=>{closeAppMenu();openSidePanel('settings-panel');};
function openAbout(updated){const d=document.getElementById('about-dialog');document.getElementById('about-tag').hidden=!updated;if(!d.open)d.showModal();document.getElementById('about-start').focus({preventScroll:true});}
document.getElementById('menu-about').onclick=()=>{closeAppMenu();openAbout(false);};
document.getElementById('about-close').onclick=document.getElementById('about-start').onclick=()=>document.getElementById('about-dialog').close();
document.querySelectorAll('.about-copy').forEach(b=>b.onclick=()=>{navigator.clipboard?.writeText(b.parentElement.querySelector('code').textContent);const t=b.textContent;b.textContent='복사됨';setTimeout(()=>{b.textContent=t;},1200);});
document.getElementById('chat-preview-form').onsubmit=event=>{event.preventDefault();const input=document.getElementById('chat-preview-input');if(!input.value.trim())return;const row=document.createElement('p');row.textContent=input.value.trim();document.getElementById('chat-preview').append(row);input.value='';};
document.getElementById('workspace-home').onclick=openWorkspaceHome;document.getElementById('goal-picker').onclick=openWorkspace;
document.querySelector('section.card[data-show="j e"] .btn.q').onclick=()=>{selectedGoalId='rotation';openWorkspace();};
document.getElementById('overview-new-goal').onclick=startNewGoal;
document.getElementById('workspace-new-goal').onclick=startNewGoal;
document.getElementById('workspace-goals-button').onclick=openWorkspace;
document.getElementById('result-to-workspace').onclick=()=>{resultReturnToList=false;document.getElementById('goal-result-dialog').close();openWorkspace();};
document.getElementById('goal-result-close').onclick=()=>document.getElementById('goal-result-dialog').close();
document.getElementById('goal-result-dialog').addEventListener('close',()=>{resultReturnToList=false;});
document.getElementById('review-main-action').onclick=()=>{if(selectedGoal().state==='completed'){showGoalResult(selectedGoalId);return;}if(!reviewOpened){reviewOpened=true;renderMergedState();return;}showGoalResult(selectedGoalId);};
function renderTeamBriefing(){
 const body=document.getElementById('team-brief-body');body.replaceChildren();
 const view=views[i][0],states=agentStates[view]||idleState;
 const stateNames={waiting:'대기 중',running:'실행 중',completed:'완료',error:'오류'};
 document.getElementById('briefing-capture').textContent=(view==='h'?'주의 필요':view==='g'?'승인 대기':view==='a'?'계획 검토':view==='d'?'완료':'진행 중')+' · 캡처 '+new Date().toLocaleString('ko-KR')+' · 읽기 전용 · 시안 데이터';
 outlineTasks.forEach((task,index)=>{
  const [state,work,attention]=states[index],started=state==='running'||state==='completed'||state==='error'||attention;
  const card=document.createElement('article');card.className='briefing-session';
  const heading=document.createElement('h4');heading.textContent=task.agent+' · '+task.id+' · '+stateNames[state];card.append(heading);
  function line(text,className=''){const node=document.createElement('p');node.textContent=text;if(className)node.className=className;card.append(node);}
  line(work,'briefing-work');
  line('세션: '+(started?'mock-'+task.agent+'-'+task.id:'미생성')+' · 모델: CLI 기본값 · 시도 '+(started?'1':'0'),'briefing-meta');
  line('프로세스: '+(state==='running'?'실행 중':state==='completed'||state==='error'?'종료':'미실행')+' · 조회 방금','briefing-meta');
  if(started){line('마지막 활동: '+(state==='running'?'출력 · 8초 전':'상태 보고 · 방금'),'briefing-meta');line(state==='error'?'bash 실행이 CLI 정책에 의해 거부되었습니다.':state==='completed'?task.title+' 결과와 근거를 기록했습니다.':task.instruction,'briefing-text');}
  line(started?'메모리 보고 (로컬 Rust, 방금): '+(state==='error'?'권한 거부와 중단 사유를 기록했습니다.':state==='completed'?task.path+' 산출물과 검증 결과를 기록했습니다.':'변경 범위 '+task.path+'와 현재 진행 상황을 기록했습니다.'):'메모리 보고: 아직 기록된 보고가 없습니다.','briefing-meta');
  if(attention)line(view==='g'?'주의: 도구 실행 승인 대기':'주의: CLI 정책에 따른 권한 거부. 실행 조건 확인이 필요합니다.','briefing-warning');
  const next=attention?(view==='g'?'사람의 승인 후 작업을 계속합니다.':'권한 설정과 작업 지시를 검토한 뒤 재시도합니다.'):state==='completed'?'후속 검증과 사람의 결과 검토를 기다립니다.':state==='running'?'작업을 계속하고 '+task.test+' 결과를 보고합니다.':index===1?'task-1 완료 후 테스트 작성을 시작합니다.':'계획 승인 후 배정된 작업을 시작합니다.';
  line('다음 단계: '+next,'briefing-next');body.append(card);
 });
 document.getElementById('briefing-approvals').textContent=view==='g'?'검토 요청: 도구 실행 · task-1 · 승인 대기':view==='c'?'검토 요청: 머지 · 통합 후보 · 승인 대기':view==='a'?'검토 요청: 계획 실행 · 전체 작업 · 승인 대기':'기록된 승인 요청이 없습니다.';
}
document.querySelectorAll('[data-team-brief]').forEach(button=>button.addEventListener('click',()=>{renderTeamBriefing();openSidePanel('team-brief-dialog');}));
document.getElementById('team-brief-close').addEventListener('click',()=>closeSidePanel('team-brief-dialog'));

document.getElementById('goal-entry').addEventListener('input',event=>{
 const view=views[i][0];if(view==='j'||view==='e')goalDrafts[view]=event.target.value;
});
document.querySelector('section.card[data-show="j e"] .btn.p').addEventListener('click',()=>{
 const value=document.getElementById('goal-entry').value.trim();
 if(!value){document.getElementById('goal-entry').focus();return;}
 const slug=document.getElementById('goal-slug-entry').value.trim();
 if(!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(slug)){document.getElementById('goal-slug-entry').focus();return;}
 if(workspaceGoals.some(goal=>goal.slug===slug&&goal.id!==selectedGoalId)){document.getElementById('goal-slug-entry').setCustomValidity('이미 사용 중인 slug입니다.');document.getElementById('goal-slug-entry').reportValidity();return;}
 document.getElementById('goal-slug-entry').setCustomValidity('');
 let goal=workspaceGoals.find(item=>item.id===selectedGoalId);
 if(!goal){goal={id:'draft-'+workspaceGoals.length,name:value,slug,state:'draft',step:1,position:'역할 확인 전',summary:'목표 저장 · 역할 확인 필요'};workspaceGoals.push(goal);selectedGoalId=goal.id;}
 goal.name=value;goal.description=value;goal.slug=slug;goal.step=1;
 document.getElementById('saved-goal-title').textContent=value;
 i=1;show();
});


document.querySelectorAll('[data-rail]').forEach(button=>button.addEventListener('click',()=>selectRail(button.dataset.rail)));
document.querySelectorAll('[data-task-index]').forEach(button=>{
 button.addEventListener('mouseenter',()=>positionStatusTip(button.querySelector('.outline-agent')));
 button.addEventListener('focus',()=>positionStatusTip(button.querySelector('.outline-agent')));
 button.addEventListener('mouseleave',()=>{if(document.activeElement!==button)document.getElementById('agent-status-tip').hidden=true;});
 button.addEventListener('blur',()=>{document.getElementById('agent-status-tip').hidden=true;});
});
document.querySelectorAll('[data-task-index]').forEach(button=>button.addEventListener('click',()=>{
 selectedOutlineTask=Number(button.dataset.taskIndex);
 if(['g','h'].includes(views[i][0])){i=3;show();}else updateTaskOutline(views[i][0]);
}));

document.querySelectorAll('[data-status-agent]').forEach(button=>{
 button.addEventListener('mouseenter',()=>positionStatusTip(button));button.addEventListener('focus',()=>positionStatusTip(button));button.addEventListener('click',()=>positionStatusTip(button));
 button.addEventListener('mouseleave',()=>{if(document.activeElement!==button)document.getElementById('agent-status-tip').hidden=true;});button.addEventListener('blur',()=>{document.getElementById('agent-status-tip').hidden=true;});
});
document.addEventListener('keydown',event=>{if(event.key==='Escape')document.getElementById('agent-status-tip').hidden=true;});
window.addEventListener('scroll',()=>{document.getElementById('agent-status-tip').hidden=true;},true);
window.addEventListener('resize',()=>{document.getElementById('agent-status-tip').hidden=true;});

(() => {
 const popup=document.getElementById('usage-popover');
 const data={claude:[64,100],codex:[73,92],'custom-cli':null};
 const errors={codex:{title:"사용량 조회 실패",message:"Claude/Codex CLI의 사용량 응답이 제한 시간 안에 도착하지 않았습니다.",detail:"quota query timed out"}};
 let active=null;
 function close(){popup.hidden=true;if(active)active.setAttribute('aria-expanded','false');}
 function open(button){
  close(); active=button; const name=button.dataset.usage,values=data[name];
  document.getElementById('usage-agent').textContent={claude:'Claude Code',codex:'Codex'}[name]||name;
  document.getElementById('usage-observed').textContent='방금 확인 · 시안 데이터';
  const error=errors[name];
  const body=document.getElementById('usage-body');
  if(error){
    body.replaceChildren();
    const box=document.createElement('div');box.className='usage-error';box.setAttribute('role','alert');
    const title=document.createElement('strong');title.textContent=error.title;
    const message=document.createElement('p');message.textContent=error.message;
    const detail=document.createElement('p');detail.textContent=error.detail;
    box.append(title,message,detail);body.append(box);
  } else {
    body.innerHTML=values?values.map((value,index)=>`<div class="usage-item ${index?'quota-long':'quota-short'}"><div class="usage-line"><span>${index?'주간':'5시간'}</span><b>${value}%</b></div><div class="usage-meter"><span style="width:${value}%"></span></div><div class="usage-reset">${index?'금요일 오전 9:00 초기화':'오후 4:30 초기화'}</div></div>`).join(''):'<p class="usage-unavailable">이 CLI는 사용량 정보를 제공하지 않습니다.<br>수치를 추정하지 않습니다.</p>';
  }
  popup.querySelector('.usage-refresh').textContent=error?'↻ 다시 조회':'↻';
  popup.querySelector('.usage-heading>.usage-reset').textContent=error?'조회 오류':'남은 사용량';
  popup.hidden=false;button.setAttribute('aria-expanded','true');
  const rect=button.getBoundingClientRect();
  popup.style.left=Math.min(innerWidth-popup.offsetWidth-12,Math.max(12,rect.right+10))+'px';
  popup.style.top=Math.min(innerHeight-popup.offsetHeight-12,Math.max(12,rect.top))+'px';
 }
 document.querySelectorAll('[data-usage]').forEach(button=>{
  const error=errors[button.dataset.usage];
  button.title=error?'사용량 조회 오류 보기':'사용량 보기';
  if(error){
    button.classList.add('error');
    button.setAttribute('aria-label',button.dataset.usage+' 사용량 조회 오류 보기');
    button.querySelectorAll('.usage-value').forEach(circle=>circle.remove());
    const mark=document.createElementNS('http://www.w3.org/2000/svg','text');
    mark.setAttribute('x','15');mark.setAttribute('y','19');mark.setAttribute('text-anchor','middle');
    mark.setAttribute('fill','currentColor');mark.setAttribute('font-size','13');mark.setAttribute('font-weight','700');
    mark.textContent='!';button.querySelector('svg').append(mark);
  }
  button.addEventListener('click',()=>{if(!popup.hidden&&active===button)close();else open(button);});
 });
 popup.querySelector('.usage-close').onclick=()=>{close();if(active)active.focus();};
 popup.querySelector('.usage-refresh').onclick=()=>{document.getElementById('usage-observed').textContent=errors[active?.dataset.usage]?'재조회 요청 예시 · 실제 CLI 호출 없음':'갱신 완료 예시 · 실제 CLI 호출 없음';};
 document.addEventListener('keydown',e=>{if(e.key==='Escape'){close();if(active)active.focus();}});
 document.addEventListener('click',e=>{if(!popup.contains(e.target)&&!e.target.closest('[data-usage]'))close();});
 window.addEventListener('resize',close);window.addEventListener('scroll',close,true);
})();
if(initialOverview)openWorkspace();
document.documentElement.dataset.designReady='true';
