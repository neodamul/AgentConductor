
// Keep each tab's header and actions outside its scrolling content.
document.querySelectorAll('.main > .card').forEach(card=>{
 const body=document.createElement('div');body.className='card-body';body.tabIndex=0;body.setAttribute('role','region');body.setAttribute('aria-label',card.querySelector('.cardhead h2').textContent+' 본문');
 const header=card.querySelector(':scope > .cardhead'),footer=card.querySelector(':scope > .dir');
 [...card.children].filter(child=>child!==header&&child!==footer).forEach(child=>body.append(child));
 card.insertBefore(body,footer);
});
const initialOverview=location.hash==='#workspace';
const views=[['j','1단계 · 워크스페이스와 목표'],['i','2단계 · 에이전트 역할'],['a','3단계 · 계획 검토'],['b','4단계 · 작업 진행'],['g','4단계 · 도구 요청 대기'],['h','4단계 · 권한 거부'],['f','5단계 · 검증'],['c','6단계 · 검토·병합'],['d','목표 완료 팝업'],['e','새 목표 · 1단계']];let i=Number(location.hash.slice(1))||0;

const idleState=[['waiting','대기 중 — 실행 중인 세션이 없습니다'],['waiting','대기 중 — 실행 중인 세션이 없습니다'],['waiting','목표를 기다리는 중']];
const agentStates={j:idleState,i:idleState,e:idleState,
a:[['waiting','대기 중 — 계획 승인을 기다립니다'],['waiting','대기 중 — 계획 승인을 기다립니다'],['completed','계획 작성 완료 — 사람의 승인을 기다립니다']],
b:[['running','실행 1 — 토큰 회전 구현 중 · 마지막 출력 8초 전'],['waiting','대기 1 — 구현 작업이 끝나면 테스트를 시작합니다'],['completed','보안 정책 노트 작성 완료']],
g:[['waiting','승인 필요 — 패키지 설치 도구 요청 대기',true],['waiting','대기 1 — 구현 완료를 기다립니다'],['completed','보안 정책 노트 작성 완료']],
h:[['completed','토큰 회전 구현 완료'],['waiting','대기 중 — 후속 검증을 기다립니다'],['error','권한 거부 — bash 실행이 CLI 정책에 의해 거부됨',true]],
f:[['completed','구현 완료 — 통합 검증 중'],['completed','테스트 작성 완료 — 통합 검증 중'],['running','실행 1 — 합쳐진 결과를 검증 중']],
c:[['completed','작업 완료 — 사람의 검토를 기다립니다'],['completed','작업 완료 — 통합 테스트 통과'],['running','실행 1 — 검토 결과 정리 중']],
d:[['completed','작업 완료 — main에 반영됨'],['completed','작업 완료 — 통합 테스트 통과'],['completed','작업 완료 — 교훈 2개 기록']]};
function positionStatusTip(button){
 const tip=document.getElementById('agent-status-tip');tip.textContent=button.dataset.statusText;tip.hidden=false;
 const rect=button.getBoundingClientRect();tip.style.left=Math.max(8,Math.min(innerWidth-tip.offsetWidth-8,rect.right+12))+'px';tip.style.top=Math.max(8,Math.min(innerHeight-tip.offsetHeight-8,rect.top))+'px';
}
function updateAgentStates(view){
 document.getElementById('agent-status-tip').hidden=true;
 const states=agentStates[view]||idleState;
 document.querySelectorAll('[data-status-agent]').forEach((button,index)=>{
 const [status,label,attention]=states[index];button.dataset.state=status;button.dataset.statusText=label;button.dataset.attention=String(!!attention);button.setAttribute('aria-label',button.dataset.statusAgent+' 상태: '+label);
 });
}


let selectedOutlineTask=0;
const outlineTasks=[
 {title:'토큰 회전 구현',agent:'claude',id:'task-1',path:'src/auth/',instruction:'리프레시 토큰을 회전하고 이전 토큰을 무효화합니다.',dependency:'없음 · 바로 시작',test:'cargo test auth'},
 {title:'재사용 탐지 테스트',agent:'codex',id:'task-2',path:'tests/auth/',instruction:'재사용된 토큰 거부와 동시 요청의 경계 조건을 검증합니다.',dependency:'task-1 완료 후 시작',test:'cargo test rotation'},
 {title:'보안 정책 노트',agent:'custom-cli',id:'task-3',path:'docs/security.md',instruction:'토큰 회전 정책과 마이그레이션 안내를 문서화합니다.',dependency:'없음 · 병렬 실행',test:'문서 검토'}
];
function selectRail(panel){


document.querySelectorAll('[data-rail]').forEach(button=>button.setAttribute('aria-selected',String(button.dataset.rail===panel)));
 document.getElementById('rail-tasks-panel').hidden=panel!=='tasks';document.getElementById('rail-agents-panel').hidden=panel!=='agents';
 const tip=document.getElementById('agent-status-tip');if(tip)tip.hidden=true;
 const quota=document.getElementById('usage-popover');if(quota)quota.hidden=true;
 document.querySelectorAll('[data-usage]').forEach(button=>button.setAttribute('aria-expanded','false'));
}
function updateTaskOutline(view){
 selectRail(['i','j','e'].includes(view)?'agents':'tasks');
 const complete=['f','c','d'].includes(view);
 const states=view==='a'?['waiting','waiting','waiting']:complete?['completed','completed','completed']:view==='h'?['completed','waiting','error']:['running','waiting','completed'];
 if(view==='g')states[0]='waiting';
 document.getElementById('outline-completed').textContent=view==='a'?'계획 검토 중':'작업 '+states.filter(status=>status==='completed').length+'/3 완료';
 const elapsed={a:'시작 전',b:'경과 06:42',g:'경과 07:10',h:'경과 07:10',f:'경과 16:20',c:'경과 18:05',d:'총 21:40'};
 document.getElementById('outline-elapsed').textContent=elapsed[view]||'시작 전';
 document.querySelectorAll('[data-task-index]').forEach((button,index)=>{
 button.setAttribute('aria-pressed',String(index===selectedOutlineTask));button.querySelector('.outline-agent').dataset.state=states[index];
 const status=view==='a'?'계획됨 · '+outlineTasks[index].dependency:states[index]==='completed'?'완료':states[index]==='error'?'권한 거부':states[index]==='running'?'실행 중':view==='g'&&index===0?'도구 승인 대기':'대기 · '+outlineTasks[index].dependency;
 const badge=button.querySelector('.outline-state-badge');badge.dataset.state=states[index];badge.textContent=view==='a'?'예정':states[index]==='completed'?'완료':states[index]==='running'?'실행 중':states[index]==='error'?'막힘':view==='g'&&index===0?'승인 대기':'대기';
  const description=outlineTasks[index].agent+' · '+status;
 button.dataset.statusText=description;button.querySelector('.outline-agent').dataset.statusText=description;
 button.querySelector('.outline-agent').dataset.attention=String(states[index]==='error'||view==='g'&&index===0);
 button.setAttribute('aria-label',outlineTasks[index].id+' '+outlineTasks[index].title+' — '+description);
 });
 renderOutlineSelection(view);
}
function renderOutlineSelection(view){
 const task=outlineTasks[selectedOutlineTask];
 const position='현재 Phase 1/1 · Task '+(selectedOutlineTask+1)+'/3';
 document.getElementById('outline-total').textContent=position;
 document.getElementById('outline-total').title='Phase: 토큰 회전 / Task: '+task.title;
 document.querySelectorAll('[data-current-position]').forEach(element=>element.textContent=position+' · '+task.title);
 const plan=document.querySelector('section.card[data-show="a"]');
 plan.querySelectorAll('.row').forEach((row,index)=>row.classList.toggle('outline-selected',index===selectedOutlineTask));
 let detail=plan.querySelector('.plan-selected-detail');if(!detail){detail=document.createElement('div');detail.className='plan-selected-detail';plan.querySelector('.phase').after(detail);}
 detail.textContent=task.id+' · '+task.title+' — '+task.instruction+' 소유 경로: '+task.path+' / 의존성: '+task.dependency+' / 검증: '+task.test;
 const card=document.querySelector('section.card[data-show="b"]');
 card.querySelector('h2').textContent=task.title;card.querySelector('.cardhead .sub').textContent=task.agent+' · '+task.id+' · '+task.path;
 document.querySelectorAll('.card[data-show="g"],.card[data-show="h"]').forEach(item=>{item.querySelector('h2').textContent=task.title;item.querySelector('.cardhead .sub').textContent=task.agent+' · '+task.id+' · '+task.path;});
 if(selectedOutlineTask===1){card.querySelector('.cardhead .pill').textContent='대기';card.querySelector('.cardhead .pill').className='pill';}
 else{card.querySelector('.cardhead .pill').textContent=selectedOutlineTask===2?'완료':'실행 중';card.querySelector('.cardhead .pill').className=selectedOutlineTask===2?'pill ok':'pill run';}
 const events=card.querySelectorAll('.ev');events.forEach(event=>event.hidden=selectedOutlineTask!==0);
 let summary=card.querySelector('.selected-task-summary');if(!summary){summary=document.createElement('div');summary.className='selected-task-summary';card.querySelector('.ttabs').after(summary);}
 summary.hidden=selectedOutlineTask===0;
 summary.textContent=selectedOutlineTask===1?'task-1 구현 완료를 기다립니다. 실행되면 테스트 결과와 로그가 여기에 표시됩니다.':'보안 정책 문서 작성을 완료했습니다. 변경과 테스트 탭에서 산출물을 확인합니다.';
 card.querySelector('.actions .note').textContent=task.agent+' · '+task.id+'에 지시를 보냅니다 · 검토는 검증 후에 열립니다';
 if(!document.getElementById('task-inspector-panel').hidden&&document.querySelector('#task-inspector-body .task-detail-content'))openTaskInspector();
}



let sidePanelOpener=null;
const panelCloseTimers=new Map();
function syncPanelBackground(){
 const visible=[...document.querySelectorAll('.slide-panel')].some(panel=>!panel.hidden);
 const open=[...document.querySelectorAll('.slide-panel')].some(panel=>panel.classList.contains('is-open'));
 const backdrop=document.getElementById('side-panel-backdrop');backdrop.hidden=!visible;backdrop.classList.toggle('visible',open);
 document.querySelector('.win>.shell').inert=visible;document.querySelector('.win>.titlebar').inert=visible;
 if(!visible&&!document.getElementById('goal-result-dialog').open&&sidePanelOpener?.isConnected&&!sidePanelOpener.closest('.slide-panel'))sidePanelOpener.focus({preventScroll:true});
}
function openSidePanel(id){
 const panel=document.getElementById(id);
 if(panel.classList.contains('is-open'))return;
 if(panelCloseTimers.has(id)){clearTimeout(panelCloseTimers.get(id));panelCloseTimers.delete(id);}
 if(![...document.querySelectorAll('.slide-panel')].some(other=>!other.hidden))sidePanelOpener=document.activeElement;
 document.querySelectorAll('.slide-panel').forEach(other=>{if(other!==panel&&!other.hidden)closeSidePanel(other.id);});
 const wasHidden=panel.hidden;panel.hidden=false;panel.inert=false;delete panel.dataset.closing;
 if(wasHidden)void panel.offsetWidth;
 panel.classList.add('is-open');syncPanelBackground();panel.querySelector('button')?.focus({preventScroll:true});
}
function closeSidePanel(id){
 if(id==='task-inspector-panel'){setTaskInspector(false);return;}
 const panel=document.getElementById(id);if(panel.hidden||panelCloseTimers.has(id))return;
 panel.classList.remove('is-open');panel.inert=true;panel.dataset.closing='true';
 if(id==='workspace-overview')workspaceVisible=false;
 if(id==='task-inspector-panel')document.getElementById('right-panel-toggle').setAttribute('aria-expanded','false');
 syncPanelBackground();
 const finish=()=>{panelCloseTimers.delete(id);if(panel.classList.contains('is-open'))return;panel.hidden=true;delete panel.dataset.closing;syncPanelBackground();};
 const delay=matchMedia('(prefers-reduced-motion: reduce)').matches?0:220;
 panelCloseTimers.set(id,setTimeout(finish,delay));
}

const workspaceGoals=[
 {id:'rotation',description:'리프레시 토큰을 쓸 때마다 새로 발급한다.\n재사용된 토큰을 탐지하면 세션을 끊는다.\n기존 로그인 흐름과 테스트 명령은 유지한다.\n결과는 변경 파일 목록과 함께 보고한다.',name:'리프레시 토큰 회전을 추가한다',slug:'refresh-token-rotation',state:'running',step:3,position:'Phase 1/1 · Task 1/3',summary:'작업 1/3 완료 · 토큰 회전 구현 중',tests:'통합 테스트 3/3 통과',commit:'3f9a1c2',lesson:'교훈 2개 · 토큰 회전 정책과 테스트 명령 기록'},
];
let pendingGoalDelete=null;
let selectedGoalId='rotation',resultGoalId='rotation',workspaceVisible=false,reviewOpened=false,resultReturnToList=false;
function selectedGoal(){return workspaceGoals.find(goal=>goal.id===selectedGoalId)||workspaceGoals[0]||{name:'새 Goal',description:'',slug:'goal-1',state:'draft'};}
function renderWorkspaceGoals(){
 const list=document.getElementById('workspace-goal-list');list.replaceChildren();if(!workspaceGoals.length){const empty=document.createElement('p');empty.className='overview-empty';empty.textContent='등록된 Goal이 없습니다. + 새 Goal로 시작하세요.';list.append(empty);}
 workspaceGoals.forEach(goal=>{
  const row=document.createElement('div');row.className='goal-list-row';
  const info=document.createElement('div'),name=document.createElement('strong'),slug=document.createElement('code'),summary=document.createElement('p');
  name.textContent=goal.name;slug.textContent=goal.slug;summary.textContent=goal.summary;name.title=goal.name;info.append(name,summary);
  // GitHub #753: the goal text keeps its line breaks, shows collapsed and expands on demand.
  if(goal.description){const text=document.createElement('p'),toggle=document.createElement('button'),label=open=>(open?'▾ 접기':'▸ 펼치기');text.className='goal-row-text';text.textContent=goal.description;toggle.type='button';toggle.className='goal-row-toggle';toggle.textContent=label(false);toggle.setAttribute('aria-expanded','false');toggle.onclick=()=>{const open=toggle.getAttribute('aria-expanded')!=='true';toggle.setAttribute('aria-expanded',String(open));text.classList.toggle('open',open);toggle.textContent=label(open);};info.append(text,toggle);}
  const state=document.createElement('div');state.className='goal-row-state';state.textContent=(goal.state==='completed'?'✓ 완료':goal.state==='draft'?'준비 중':'● 실행 중');
  const position=document.createElement('small');position.style.display='block';position.style.color='var(--soft)';position.textContent=goal.position;state.append(position);
  const action=document.createElement('button');action.className='btn q';action.textContent=goal.state==='completed'?'결과 보기':'이어서 진행';
  action.onclick=()=>{if(goal.state==='completed')showGoalResult(goal.id);else enterGoal(goal);};
  const actions=document.createElement('div');actions.className='goal-row-actions';
  const folder=document.createElement('a');folder.className='goal-folder';
  const project=document.querySelector('input[aria-label="작업 디렉터리"]').value.trim().replace(/\/+$/,'');
  const folderUrl=new URL('file:///');folderUrl.pathname=project+'/'+goal.slug+'/';
  folder.href=folderUrl.href;folder.target='_blank';folder.rel='noopener';folder.title='Goal 작업 폴더 열기\n'+project+'/'+goal.slug;folder.setAttribute('aria-label',goal.name+' 작업 폴더 열기');
  folder.innerHTML='<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 7V5a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z"/><path d="M3 8h18"/></svg>';
  const trash=document.createElement('button');trash.type='button';trash.className='goal-folder goal-trash';trash.title='Goal 폴더 영구 삭제';trash.setAttribute('aria-label',goal.name+' 영구 삭제');
  trash.innerHTML='<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 6h18M9 6V4h6v2M5 6l1 14h12l1-14M10 10v6M14 10v6"/></svg>';
  trash.onclick=()=>{pendingGoalDelete=goal.id;document.getElementById('delete-goal-name').textContent=goal.name;document.getElementById('delete-goal-path').textContent=project+'/'+goal.slug;document.getElementById('delete-goal-dialog').showModal();};
  actions.append(action,folder);row.append(trash,info,state,actions);list.append(row);
 });
}

function openWorkspaceHome(){
 closeSidePanel('workspace-overview');closeSidePanel('team-brief-dialog');workspaceVisible=false;if(selectedGoalId!=='draft')goalDrafts.j=selectedGoal().description||selectedGoal().name;i=selectedGoalId==='draft'?9:0;show();
}
function refreshWorkspaceGoalSection(){
 const draft=selectedGoalId==='draft',goal=selectedGoal();
 document.querySelectorAll('[data-goal-count]').forEach(badge=>badge.textContent=workspaceGoals.length);
 document.getElementById('workspace-current-goal').textContent=draft?'새 Goal 작성':goal.name;
 document.getElementById('workspace-current-goal-state').textContent=draft?'아직 저장하지 않음':goal.state==='completed'?'완료':goal.state==='draft'?'준비 중':'진행 중';
}

function openWorkspace(){
 const wasOpen=document.getElementById('workspace-overview').classList.contains('is-open');
 workspaceVisible=true;if(!wasOpen)renderWorkspaceGoals();openSidePanel('workspace-overview');
 document.getElementById('label').textContent='워크스페이스 · 목표 목록';location.hash='workspace';
}
function enterGoal(goal){
 selectedGoalId=goal.id;workspaceVisible=false;closeSidePanel('workspace-overview');
 document.getElementById('saved-goal-title').textContent=goal.name;
 i=goal.state==='completed'?7:goal.step;show();
}
function startNewGoal(){
 goalDrafts.e='';
 document.getElementById('goal-result-dialog').close();closeSidePanel('workspace-overview');workspaceVisible=false;selectedGoalId='draft';
 let number=workspaceGoals.length+1;while(workspaceGoals.some(goal=>goal.slug==='goal-'+number))number++;document.getElementById('goal-slug-entry').value='goal-'+number;i=9;show();
}
function showGoalResult(id){
 resultReturnToList=document.getElementById('workspace-overview').classList.contains('is-open');
 const goal=workspaceGoals.find(item=>item.id===id)||workspaceGoals[0];if(!goal)return;resultGoalId=goal.id;
 if(id==='rotation'){goal.state='completed';goal.step=7;goal.position='Phase 1/1 완료';goal.summary='main 반영 · 통합 테스트 3/3 통과';}
 document.getElementById('goal-result-title').textContent=goal.name+' — 완료';document.getElementById('result-slug').textContent=goal.slug;
 const rotation=goal.id==='rotation';
 const summary=document.getElementById('result-summary');summary.replaceChildren();
 const text=document.createElement('p');text.textContent=rotation?'리프레시 토큰 회전과 재사용 탐지를 구현하고, 검토한 변경을 main에 반영했습니다.':'경량 상태 확인 API와 의존 서비스 실패 검사를 구현하고 main에 반영했습니다.';summary.append(text);
 const commit=document.createElement('p');commit.className='result-commit';commit.textContent='승인 완료 · main 반영 · 커밋 '+goal.commit;summary.append(commit);const directory=document.createElement('p');directory.className='result-commit';directory.textContent='반영 위치: USERHOME/Development/auth-service';summary.append(directory);
 const stats=document.createElement('div');stats.className='result-stats';
 [[rotation?'4':'3','변경 파일'],[rotation?'3/3':'8/8','테스트 통과'],[rotation?'2':'1','기록한 교훈']].forEach(([value,label])=>{const tile=document.createElement('div'),number=document.createElement('strong'),caption=document.createElement('span');number.textContent=value;caption.textContent=label;tile.append(number,caption);stats.append(tile);});summary.append(stats);
 document.getElementById('result-lesson').textContent='';
 const detail=document.getElementById('result-detail');detail.replaceChildren();detail.hidden=false;
 const section=(heading,rows)=>{const title=document.createElement('h3');title.textContent=heading;detail.append(title);const list=document.createElement('ul');rows.forEach(value=>{const item=document.createElement('li');item.textContent=value;list.append(item);});detail.append(list);};
 section('변경 파일',rotation?['src/auth/refresh.rs · +84 / −12','src/auth/store.rs · +21 / −3','tests/auth/rotation.rs · +66 / −0','docs/security.md · +18 / −2']:['src/health.rs','tests/health.rs','docs/health.md']);
 section('검증 결과',rotation?['cargo test auth · 통과','cargo test rotation · 통과','보안 정책 문서 검토 · 통과']:['상태 확인 및 의존 서비스 실패 테스트 · 8/8 통과']);
 section('남긴 교훈',rotation?['테스트 명령은 cargo test auth로 범위를 좁혀 실행한다.','refresh.rs와 store.rs의 토큰 상태 변경을 함께 검토한다.']:['의존 서비스 실패 시 반환하는 상태와 응답 규칙을 문서화한다.']);
 const dialog=document.getElementById('goal-result-dialog');if(!dialog.open)dialog.showModal();
 renderMergedState();
}
function renderMergedState(){
 const goal=selectedGoal();const merged=goal.state==='completed';
 document.getElementById('review-run-state').textContent=merged?'완료':'승인 대기';
 document.getElementById('review-goal-title').textContent=merged?'검토한 결과가 main에 반영되었습니다':'변경을 검토하고 승인합니다';
 if(merged){updateAgentStates('d');updateTaskOutline('d');}
 const button=document.getElementById('review-main-action');button.textContent=merged?'완료 결과 보기':reviewOpened?'승인하고 머지 (시안)':'변경 검토';
 document.querySelectorAll('section.card[data-show="c"] .btn:not(#review-main-action)').forEach(control=>control.disabled=merged);
}
const goalDrafts={j:'리프레시 토큰을 쓸 때마다 새로 발급하고, 재사용된 토큰을 탐지하면 세션을 끊는다.',e:'로그인 실패가 5번 넘으면 계정을 15분 잠근다. 잠금 해제 API와 테스트를 추가한다.'};
function show(){const popupResult=views[i][0]==='d';const v=popupResult?'c':views[i][0];
 if(!workspaceVisible)closeSidePanel('workspace-overview');
 document.getElementById('current-goal-slug').textContent=selectedGoalId==='draft'?'새 목표 · slug는 저장 시 확정':selectedGoal().slug;
 if(v==='j')document.getElementById('goal-slug-entry').value=selectedGoal().slug;
 refreshWorkspaceGoalSection();renderMergedState();if(v==='j'||v==='e')document.getElementById('goal-entry').value=goalDrafts[v];if(v==='g')selectedOutlineTask=0;if(v==='h')selectedOutlineTask=2;document.querySelectorAll('[data-show]').forEach(e=>e.style.display=e.dataset.show.split(' ').includes(v)?'':'none');
document.querySelectorAll('[data-cls]').forEach(e=>{e.className='tab';const m=e.dataset.cls.split(' ').map(x=>x.split(':')).find(x=>x[0]===v);if(m)e.classList.add(m[1]);const n=e.querySelector('[data-num]');n.textContent=e.classList.contains('done')?'✓':n.dataset.num;});
document.querySelectorAll('.agent').forEach(e=>e.classList.toggle('sel',(e.dataset.v||'').split(' ').includes(v)));
updateAgentStates(v);
updateTaskOutline(v);
 document.querySelectorAll('[data-team-brief]').forEach(button=>button.disabled=['j','i','e'].includes(v));
 document.getElementById('saved-goal-title').title=document.getElementById('saved-goal-title').textContent;
document.getElementById('label').textContent=`${i+1} / ${views.length}  ${views[i][1]}`;location.hash=i;if(selectedGoal().state==='completed'&&v==='c'){const tab=document.querySelector('.tabs .tab[data-step="6"]');tab.classList.add('done');tab.querySelector('.num').textContent='✓';}if(popupResult)showGoalResult('rotation');}
prev.onclick=()=>{workspaceVisible=false;i=(i+views.length-1)%views.length;show()};next.onclick=()=>{workspaceVisible=false;i=(i+1)%views.length;show()};show();





document.querySelectorAll('[data-panel-close]').forEach(button=>button.onclick=()=>closeSidePanel(button.dataset.panelClose));
document.getElementById('side-panel-backdrop').onclick=()=>document.querySelectorAll('.slide-panel').forEach(panel=>closeSidePanel(panel.id));
document.addEventListener('keydown',event=>{
 if(document.querySelector('dialog[open]')||!document.getElementById('assignment-menu').hidden)return;
 const panel=[...document.querySelectorAll('.slide-panel')].find(item=>item.classList.contains('is-open'));if(!panel)return;
 if(event.key==='Escape'){event.preventDefault();closeSidePanel(panel.id);}
 if(event.key==='Tab'){const controls=[...panel.querySelectorAll('button,input,select,textarea,a[href]')].filter(control=>!control.disabled);const first=controls[0],last=controls.at(-1);if(event.shiftKey&&document.activeElement===first){event.preventDefault();last?.focus({preventScroll:true});}else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first?.focus({preventScroll:true});}}
});


document.getElementById('delete-goal-dialog').addEventListener('close',event=>{
 if(event.target.returnValue==='delete'&&pendingGoalDelete){const index=workspaceGoals.findIndex(goal=>goal.id===pendingGoalDelete);if(index>=0)workspaceGoals.splice(index,1);if(selectedGoalId===pendingGoalDelete){selectedGoalId=workspaceGoals[0]?.id||'draft';i=workspaceGoals.length?0:9;goalDrafts[workspaceGoals.length?'j':'e']=workspaceGoals[0]?.description||'';show();}refreshWorkspaceGoalSection();renderWorkspaceGoals();document.getElementById('overview-new-goal').focus({preventScroll:true});}pendingGoalDelete=null;
});
