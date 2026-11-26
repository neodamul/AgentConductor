// Goal history uses sample records in the design preview. The normal setup keeps its single fixture.
function previewGoalHistoryFixtures() {
 const names=['로그인 오류 메시지 정리','세션 만료 정책 문서화','인증 로그 필드 추가','세션 해지 테스트 추가','로그인 감사 이벤트 확인','인증 응답 코드 검토','관리자 세션 만료 테스트','테스트 계정 설정 정리','토큰 오류 대응 문서 작성','인증 에러 현황 정리','로그인 화면 검증'];
 return names.map((name,index)=>{
  const number=(index+1)*2;
  const references=[['jira','AUTH-'+number]];
  if(index%3===0)references.push(['confluence',String(9876543210+number)]);
  return {
   id:'history-'+number,name,slug:'auth_goal_'+number+'__refs_jira_auth_'+number+
    (references[1]?'_confluence_'+references[1][1]:''),
   references,state:'draft',step:1,position:'계획 전',summary:'역할 확인 대기'
  };
 });
}

const previewGoalHistoryControls={
 search:document.getElementById('workspace-goal-search'),
 sort:document.getElementById('workspace-goal-sort'),
 previous:document.getElementById('workspace-goal-prev'),
 next:document.getElementById('workspace-goal-next'),
 status:document.getElementById('workspace-goal-page'),
 nav:document.getElementById('workspace-goal-pagination')
};
let previewGoalPage=1;
const previewGoalOrder=new Intl.Collator('en',{numeric:true,sensitivity:'base'});
function previewGoalHistory(goals) {
 const {search,sort,previous,next,status,nav}=previewGoalHistoryControls;
 const terms=search.value.trim().toLowerCase().replace(/:\s+/g,':').split(/\s+/).filter(Boolean);
 const matches=goals.filter(goal=>{
  if(!terms.length)return true;
  const fields=[goal.name,goal.slug,goal.description||'',...(goal.references||[]).map(ref=>ref.join(':'))];
  return terms.every(term=>fields.some(field=>field.toLowerCase().includes(term)));
 });
 if(sort.value!=='saved'){
  const [kind,order]=sort.value.split('-');
  const compare=(a,b)=>(order==='desc'?-1:1)*previewGoalOrder.compare(a,b);
  const ids=new Map(matches.map(goal=>[goal,(goal.references||[]).filter(ref=>ref[0]===kind).map(ref=>ref[1]).sort(compare)[0]]));
  matches.sort((a,b)=>{
   const left=ids.get(a),right=ids.get(b);
   if(left===undefined||right===undefined)return Number(left===undefined)-Number(right===undefined);
   return compare(left,right);
  });
 }
 const pageSize=10,pages=Math.max(1,Math.ceil(matches.length/pageSize));
 previewGoalPage=Math.max(1,Math.min(previewGoalPage,pages));
 nav.hidden=pages===1;
 previous.disabled=previewGoalPage===1;
 next.disabled=previewGoalPage===pages;
 status.textContent=previewGoalPage+' / '+pages+' 페이지';
 return matches.slice((previewGoalPage-1)*pageSize,previewGoalPage*pageSize);
}

const {search:historySearch,sort:historySort,previous:historyPrev,next:historyNext}=previewGoalHistoryControls;
historySearch.oninput=historySort.onchange=()=>{previewGoalPage=1;renderWorkspaceGoals();};
historyPrev.onclick=()=>{previewGoalPage--;renderWorkspaceGoals();};
historyNext.onclick=()=>{previewGoalPage++;renderWorkspaceGoals();};
