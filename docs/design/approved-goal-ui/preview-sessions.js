// Agent sessions: each agent card counts its sessions; the dialog lists them with state, Goal/task and times,
// stops running ones and cleans up finished, failed or stale ones after a confirmation.
const agentSessions={
 claude:[
  {id:'claude-t1',task:'T1 토큰 회전 구현',goal:'리프레시 토큰 회전을 추가한다',state:'running',started:'21:02',last:'방금'},
  {id:'claude-plan',task:'계획 작성 · 코디네이터',goal:'리프레시 토큰 회전을 추가한다',state:'done',started:'20:40',last:'20:52'},
  {id:'claude-old',task:'T2 세션 만료 처리',goal:'로그인 잠금을 추가한다',state:'stale',started:'어제 18:10',last:'어제 18:40',note:'프로세스 없음 · 로그 보존'}
 ],
 codex:[
  {id:'codex-t2',task:'T2 재사용 탐지 테스트',goal:'리프레시 토큰 회전을 추가한다',state:'waiting',started:'21:05',last:'1분 전',note:'도구 승인 대기'},
  {id:'codex-t3',task:'T3 통합 테스트',goal:'로그인 잠금을 추가한다',state:'failed',started:'어제 17:20',last:'어제 17:31',note:'종료 코드 1'}
 ],
 'custom-cli':[]
};
const sessionLabels={running:'실행 중',waiting:'대기',done:'끝남',failed:'실패',stale:'정체',stopped:'중지됨'};
const sessionsDialog=document.getElementById('sessions-dialog'),sessionsField=name=>document.getElementById('sessions-'+name);
let sessionsAgent=null,sessionsPending=null;
const finishedSession=session=>['done','failed','stale','stopped'].includes(session.state);
function renderSessionButtons(){
 document.querySelectorAll('[data-sessions]').forEach(button=>{
  const list=agentSessions[button.dataset.sessions]||[],running=list.filter(s=>s.state==='running'||s.state==='waiting').length;
  button.replaceChildren('세션',Object.assign(document.createElement('span'),{className:'session-count',textContent:String(list.length)}));
  button.classList.toggle('has-running',running>0);
  button.setAttribute('aria-label',(agentDisplay[button.dataset.sessions]||button.dataset.sessions)+' 세션 '+list.length+'개'+(running?' · 진행 중 '+running+'개':''));
 });
}
function renderSessions(){
 const list=agentSessions[sessionsAgent]||[];
 sessionsField('title').textContent=(agentDisplay[sessionsAgent]||sessionsAgent)+' · 세션';
 const count=state=>list.filter(s=>s.state===state).length;
 sessionsField('summary').textContent=list.length?'실행 중 '+count('running')+' · 대기 '+count('waiting')+' · 끝남 '+(count('done')+count('stopped'))+' · 실패 '+count('failed')+' · 정체 '+count('stale'):'';
 const rows=sessionsField('list');rows.replaceChildren();
 if(!list.length){const empty=document.createElement('p');empty.className='panel-note';empty.textContent='이 에이전트의 세션이 없습니다. 작업을 배정하면 여기에 나타납니다.';rows.append(empty);}
 list.forEach(session=>{
  const row=document.createElement('div');row.className='session-row';row.setAttribute('role','listitem');row.dataset.session=session.id;
  const badge=document.createElement('span');badge.className='session-state '+session.state;badge.textContent=sessionLabels[session.state];
  const info=document.createElement('div'),task=document.createElement('strong'),meta=document.createElement('small');
  task.textContent=session.task;task.title=session.task;meta.textContent=session.goal+' · 시작 '+session.started+' · 최근 '+session.last+(session.note?' · '+session.note:'');meta.title=meta.textContent;info.append(task,meta);
  const action=document.createElement('button');action.type='button';action.className='btn q';
  if(finishedSession(session)){action.textContent='정리';action.title='이 세션 정리';action.onclick=()=>askCleanup([session.id]);}
  else{action.textContent='중지';action.title='이 세션 중지';action.onclick=()=>{session.state='stopped';session.last='방금';session.note='사용자가 중지함';renderSessions();renderSessionButtons();};}
  row.append(badge,info,action);rows.append(row);
 });
 const finished=list.filter(finishedSession).length;
 sessionsField('clean-finished').disabled=!finished;sessionsField('clean-finished').textContent='끝난 세션 정리'+(finished?' ('+finished+')':'');
}
function askCleanup(ids){
 sessionsPending=ids;
 sessionsField('confirm-text').textContent=(ids.length===1?'이 세션':'끝난 세션 '+ids.length+'개')+'을 정리합니다. 세션 기록과 작업 폴더(worktree)를 지우고, 로그는 캐시 정리 전까지 남깁니다. 되돌릴 수 없습니다.';
 sessionsField('confirm').hidden=false;sessionsField('confirm-cancel').focus();
}
function openSessions(agent){sessionsAgent=agent;sessionsPending=null;sessionsField('confirm').hidden=true;renderSessions();if(!sessionsDialog.open)sessionsDialog.showModal();}
sessionsField('confirm-cancel').onclick=()=>{sessionsPending=null;sessionsField('confirm').hidden=true;};
sessionsField('confirm-ok').onclick=()=>{
 agentSessions[sessionsAgent]=(agentSessions[sessionsAgent]||[]).filter(session=>!sessionsPending.includes(session.id));
 sessionsPending=null;sessionsField('confirm').hidden=true;renderSessions();renderSessionButtons();
};
sessionsField('clean-finished').onclick=()=>askCleanup((agentSessions[sessionsAgent]||[]).filter(finishedSession).map(session=>session.id));
sessionsField('close').onclick=()=>sessionsDialog.close();
document.querySelectorAll('[data-sessions]').forEach(button=>button.onclick=()=>openSessions(button.dataset.sessions));
renderSessionButtons();
