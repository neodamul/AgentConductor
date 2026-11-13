// #943: after a phase is merged the coordinator re-plans the next phase from the merged result.
// The plan card shows what changed against the first plan; "Merge automatically" skips only the merge approval.
let replanPhase=null;
const firstPlanRows=document.getElementById('plan-rows').innerHTML;
const replanRows=[
 {n:1,t:'토큰 폐기 API',s:['codex · src/auth/revoke.rs','cargo test revoke'],change:'changed',reason:'처음 계획: tests/auth/ 테스트만 작성. Phase 1에서 회전 토큰 저장소가 src/auth/store.rs로 정해져 폐기 API 구현까지 맡깁니다.'},
 {n:2,t:'저장소 마이그레이션',s:['claude · migrations/','1번 이후'],change:'added',reason:'Phase 1이 토큰 테이블에 family_id 열을 추가해 기존 데이터를 옮기는 작업이 필요합니다.'},
 {n:3,t:'감사 로그 노트',s:['custom-cli · docs/audit.md','—'],change:'removed',reason:'Phase 1의 보안 정책 노트(docs/security.md)가 감사 로그 항목을 이미 다룹니다.'}
];
const changeLabel={changed:'변경',added:'추가',removed:'제외'};
function renderPlanCard(){
 const card=document.getElementById('plan-card');if(!card)return;
 const rows=document.getElementById('plan-rows'),next=replanPhase;
 document.getElementById('plan-eyebrow').textContent=next?'3단계 · 계획 · Phase '+next.number+'/'+next.total:'3단계 · 계획';
 document.getElementById('plan-heading').textContent=next?'Phase '+(next.number-1)+' 결과로 Phase '+next.number+'을 다시 계획했습니다':'계획을 검토하고 실행을 시작합니다';
 document.getElementById('plan-sub').textContent=next?'코디네이터가 Phase '+(next.number-1)+' 병합 결과(변경 4개 파일 · 통합 테스트 통과)를 보고 처음 계획을 고쳤습니다. 달라진 작업에는 표시와 이유가 붙습니다.':'코디네이터가 목표와 저장소를 분석해 작업 3개를 제안했습니다. 작업을 펼쳐 지시와 담당을 고칠 수 있습니다.';
 if(!next){rows.innerHTML=firstPlanRows;}else{
  rows.replaceChildren(...replanRows.map(item=>{
   const row=document.createElement('div');row.className='row';row.dataset.change=item.change;
   const n=document.createElement('span');n.className='n';n.textContent=item.n;
   const t=document.createElement('span');t.className='t';t.textContent=item.t;
   const tag=document.createElement('span');tag.className='plan-tag plan-change';tag.dataset.change=item.change;tag.textContent=changeLabel[item.change];t.append(tag);
   row.append(n,t,...item.s.map(text=>{const s=document.createElement('span');s.className='s';s.textContent=text;return s;}));
   const fold=document.createElement('span');fold.className='s fold';fold.textContent='▸ 펼치기';row.append(fold);const reason=document.createElement('span');reason.className='plan-change-reason';reason.textContent=item.reason;row.append(reason);
   return row;
  }));
 }
 document.querySelectorAll('[data-current-position]').forEach(e=>e.textContent=next?'현재 Phase '+next.number+'/'+next.total+' · 작업 2개 · 1개 제외':'현재 Phase 1/1 · Task 1/3');
 renderAutoMergeNote();
}
function renderAutoMergeNote(){
 const on=document.getElementById('auto-merge').checked;
 document.getElementById('plan-approve-note').textContent=on?'검증을 통과하면 승인 없이 병합하고 다음 Phase를 다시 계획합니다. 검증 실패·범위 검토·도구 승인은 멈추고 기다립니다.':'승인하면 같은 화면에서 4단계 작업으로 넘어갑니다.';
}
document.getElementById('auto-merge').onchange=renderAutoMergeNote;
// When the re-plan changes nothing the plan stop is skipped and the next phase starts at once.
function previewReplan(number,total,unchanged){
 if(number&&unchanged){i=3;show();document.getElementById('run-meta').replaceChildren(Object.assign(document.createElement('span'),{textContent:'Phase '+number+'/'+total+' · Phase '+(number-1)+' 결과로 다시 검토했고 계획 변경이 없어 바로 시작했습니다'}));return;}
 replanPhase=number?{number,total}:null;renderPlanCard();
}
renderPlanCard();
// Any other navigation shows the first plan again; previewReplan runs after show() on the next-phase path.
const showFirstPlan=show;show=function(){replanPhase=null;showFirstPlan();document.getElementById('run-meta').replaceChildren();renderPlanCard();};
