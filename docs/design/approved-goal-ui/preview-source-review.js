// GitHub #903: document preview window — a guide and one progress card per Goal reference.
// Fixture progress only; the app renders the same cards from the snapshot's mcp_progress and mcp_result.
const sourceText={
 stage_waiting:'대기',stage_selection:'도구 선택 중',stage_reading:'원문 조회 중',stage_summary_wait:'요약 대기',stage_summary:'요약 중',stage_done:'요약 완료',stage_review:'검토',stage_failed:'실패',
 step_select:'도구 선택',step_read:'원문 조회',step_summary:'요약',step_review:'검토',title_pending:'제목은 조회 후 표시',title_missing:'제목 없음',
 log_coordinator:'코디네이터 시작 ({detail})',log_inspect:'스키마 확인: {detail}',log_selected:'도구 선택: {detail}',log_helper:'보조 조회: {detail}',log_invalid_arguments:'인자 오류 · 다시 선택: {detail}',log_mcp_call:'MCP 호출 · 응답 대기 중 (제한 {detail}초)',log_read_done:'원문 조회 완료 · {detail}',log_summary:'요약 시작 ({detail})',log_summary_done:'요약 완료',log_carried:'이전 조회 결과 사용',log_failed:'실패',
 full_source_size:'원문 보기 ({size})',meta_running:'문서 {total}개 중 {done}개 완료 · 전체 {elapsed}',meta_done:'완료 {done} · 실패 {failed} · 전체 {elapsed}',
 status_running:'문서 조회 {done}/{total} · {reference} {stage} · {elapsed}',provenance:'{server} · {tool} · 버전 {version} · SHA-256 {sha}'
};
const sourceT=(key,vars={})=>sourceText[key].replace(/\{(\w+)\}/g,(_,k)=>vars[k]??'');
// Shared with the app's mcp.js: duration, step index, card rendering.
function sourceDuration(ms){const s=Math.max(0,Math.round(ms/1000));return s<60?s+'초':Math.floor(s/60)+'분 '+String(s%60).padStart(2,'0')+'초';}
function sourceSize(bytes){return bytes<1024?bytes+' B':Math.round(bytes/1024)+' KB';}
function sourceFailedStep(doc){const events=doc.log.map(e=>e.event);return events.includes('summary')?2:events.includes('mcp_call')?1:0;}
function sourceCard(doc,{now,record,summary,review,t,size}){
 const mk=(tag,cls,text)=>{const n=document.createElement(tag);if(cls)n.className=cls;if(text!==undefined)n.textContent=text;return n;};
 const running=['selection','reading','summary'].includes(doc.stage),failed=doc.stage==='failed';
 const card=mk('section','source-doc'+(running?' running':'')+(failed?' failed':''));card.dataset.reference=doc.reference;card.dataset.stage=doc.stage;
 const head=mk('div','source-doc-head');
 if(review){const pick=mk('input');pick.type='checkbox';pick.value=doc.reference;pick.checked=!!record;pick.disabled=!record||doc.reference.startsWith('jira:');pick.setAttribute('aria-label',doc.reference);head.append(pick);}
 const [kind,id]=doc.reference.split(':');
 const ref=mk('span','source-ref',(kind==='jira'?'Jira':'Confluence')+':'+id);ref.title=ref.textContent;head.append(ref);
 const title=doc.title||record?.title;head.append(mk('strong','source-title'+(title?'':' pending'),title||t(failed?'title_missing':'title_pending')));head.querySelector('.source-title').title=title||'';
 const label=failed?'stage_failed':doc.stage==='done'?(review?'stage_review':'stage_done'):'stage_'+doc.stage;
 head.append(mk('span','source-state '+(failed?'failed':doc.stage==='waiting'?'waiting':['done','summary_wait'].includes(doc.stage)?'ready':'running'),t(label)));
 head.append(mk('span','source-time',doc.started_ms?sourceDuration((doc.ended_ms||now)-doc.started_ms):'—'));
 card.append(head);
 const current={waiting:-1,selection:0,reading:1,summary_wait:2,summary:2,done:3}[doc.stage]??sourceFailedStep(doc);
 const steps=mk('ol','source-steps');['step_select','step_read','step_summary','step_review'].forEach((key,i)=>steps.append(mk('li','source-step'+(failed?(i<current?' done':i===current?' failed':''):(i<current?' done':i===current?' current':'')),t(key))));
 card.append(steps);
 if(running||failed){const lines=doc.log.map(e=>{const at=Math.max(0,Math.round((e.at_ms-doc.started_ms)/1000));return String(Math.floor(at/60)).padStart(2,'0')+':'+String(at%60).padStart(2,'0')+' '+t('log_'+e.event,{detail:e.event==='read_done'?size(Number(e.detail)):e.detail});});if(lines.length)card.append(mk('div','source-log',lines.join('\n')));}
 if(failed&&doc.error)card.append(mk('p','source-error',doc.error));
 if(record){
  if(summary)card.append(mk('div','source-summary',summary.text));
  const full=mk('details','source-full');full.append(mk('summary','',t('full_source_size',{size:size(record.body.length)})));full.ontoggle=()=>{if(full.open&&full.childNodes.length===1)full.append(mk('pre','source-body',record.body));};card.append(full);
  card.append(mk('p','source-provenance',t('provenance',{server:record.server,tool:record.tool,version:record.version||'—',sha:record.sha256.slice(0,8)})));
 }
 return card;
}
function sourceMeta(progress,finished,now,t){const docs=progress.documents,done=docs.filter(d=>d.stage==='done'||!finished&&d.stage==='summary_wait').length,failed=docs.filter(d=>d.stage==='failed').length,end=finished?Math.max(...docs.map(d=>d.ended_ms||progress.started_ms)):now;const elapsed=sourceDuration(end-progress.started_ms);return finished?t('meta_done',{done,failed,elapsed}):t('meta_running',{total:docs.length,done,elapsed});}
// Fixture: three references at a fixed moment of the lookup and after it ended.
const sourceStart=1_700_000_000_000,sourceNow=sourceStart+239_000;
const sourceRunning={started_ms:sourceStart,documents:[
 {reference:'jira:EMM-52599',stage:'summary_wait',title:'결제 실패 시 재시도 정책 개선',started_ms:sourceStart,ended_ms:sourceStart+112_000,log:[{at_ms:sourceStart,event:'coordinator',detail:'custom-cli'},{at_ms:sourceStart+38_000,event:'selected',detail:'atlassian · jira_get_issue {"issueKey":"EMM-52599"} · 1/8'},{at_ms:sourceStart+40_000,event:'mcp_call',detail:'300'},{at_ms:sourceStart+112_000,event:'read_done',detail:'34816'}]},
 {reference:'confluence:53553565',stage:'reading',title:'결제 모듈 아키텍처 v3',started_ms:sourceStart+112_000,log:[{at_ms:sourceStart+112_000,event:'coordinator',detail:'custom-cli'},{at_ms:sourceStart+153_000,event:'selected',detail:'atlassian · confluence_get_page {"pageId":"53553565"} · 1/8'},{at_ms:sourceStart+155_000,event:'mcp_call',detail:'300'}]},
 {reference:'jira:EMM-52611',stage:'waiting',log:[]}
]};
const sourceFinished={started_ms:sourceStart,documents:[
 {...sourceRunning.documents[0],stage:'done',ended_ms:sourceStart+151_000},
 {...sourceRunning.documents[1],stage:'done',ended_ms:sourceStart+340_000},
 {reference:'jira:EMM-52611',stage:'failed',started_ms:sourceStart+340_000,ended_ms:sourceStart+640_000,error:'EMM-52611: 원문 조회 단계가 300초 안에 끝나지 않았습니다. 문서는 승인되지 않았습니다. 로컬 CLI 로그를 확인하고 다시 시도하세요.',log:[{at_ms:sourceStart+340_000,event:'coordinator',detail:'custom-cli'},{at_ms:sourceStart+381_000,event:'selected',detail:'atlassian · jira_get_issue {"issueKey":"EMM-52611"} · 1/8'},{at_ms:sourceStart+383_000,event:'mcp_call',detail:'300'},{at_ms:sourceStart+640_000,event:'failed',detail:''}]}
]};
const sourcePreview={records:[
 {reference:{kind:'jira',id:'EMM-52599'},server:'atlassian',tool:'jira_get_issue',title:'결제 실패 시 재시도 정책 개선',version:'12',sha256:'3f9a6c1e0b7d',body:'x'.repeat(34816)},
 {reference:{kind:'confluence',id:'53553565'},server:'atlassian',tool:'confluence_get_page',title:'결제 모듈 아키텍처 v3',version:'41',sha256:'a1c4e9f07d2b',body:'x'.repeat(120832)}
],summaries:{'jira:EMM-52599':{text:'재시도는 최대 3회, 지수 백오프(2·4·8초)로 바꾸고 멱등 키가 없는 결제는 재시도하지 않는다. 완료 조건: 실패 사유별 알림, 대시보드 지표 추가.'},'confluence:53553565':{text:'결제 게이트웨이 어댑터 계층과 재시도 큐를 분리한다. 외부 PG 3곳에 공통 인터페이스를 둔다.'}}};
function renderSourceReview(mode){
 const finished=mode==='review',progress=finished?sourceFinished:sourceRunning,now=finished?sourceStart+640_000:sourceNow;
 const records=new Map(sourcePreview.records.map(r=>[r.reference.kind+':'+r.reference.id,r]));
 document.getElementById('source-docs').replaceChildren(...progress.documents.map(doc=>sourceCard(doc,{now,record:finished?records.get(doc.reference):null,summary:finished?sourcePreview.summaries[doc.reference]:null,review:finished,t:sourceT,size:sourceSize})));
 document.getElementById('source-meta').textContent=sourceMeta(progress,finished,now,sourceT);
 document.getElementById('source-retry').hidden=!(finished&&progress.documents.some(d=>d.stage==='failed'));
 document.getElementById('source-apply').disabled=!finished||progress.documents.some(d=>d.stage==='failed'&&d.reference.startsWith('jira:'));
 const running=progress.documents.find(d=>['selection','reading','summary'].includes(d.stage));
 document.getElementById('source-status').textContent=finished||!running?'':sourceT('status_running',{done:progress.documents.filter(d=>d.stage==='done'||d.stage==='summary_wait').length,total:progress.documents.length,reference:running.reference,stage:sourceT('stage_'+running.stage),elapsed:sourceDuration(now-progress.started_ms)});
}
function openSourceReview(mode='running'){renderSourceReview(mode);const d=document.getElementById('source-review');if(!d.open)d.showModal();}
// #906 adds the Goal's Preview documents button; the status line follows it.
const sourcePreviewButton=document.getElementById('workspace-preview-documents');sourcePreviewButton.after(document.getElementById('source-status'));
sourcePreviewButton.addEventListener('click',()=>openSourceReview('running'));
document.getElementById('source-close').onclick=()=>document.getElementById('source-review').close();
document.getElementById('source-retry').onclick=()=>openSourceReview('running');
{const mode=new URLSearchParams(location.search).get('sources');if(mode)openSourceReview(mode);}
// GitHub #907: a Goal holds at most five distinct references; adding stops at five and a sixth blocks preview.
const SOURCE_LIMIT=5;
function sourceLimit(){
 const refs=new Set([...document.getElementById('goal-entry').value.matchAll(/\[(Jira|Confluence):([^\]]+)\]/g)].map(m=>m[1].toLowerCase()+':'+m[2]));
 const count=refs.size,full=count>=SOURCE_LIMIT,over=count>SOURCE_LIMIT,status=document.getElementById('source-status');
 for(const id of ['source-jira','source-confluence']){const b=document.getElementById(id);b.disabled=full;b.title=full?'Goal에는 참조 문서를 5개까지 넣을 수 있습니다.':'';}
 sourcePreviewButton.disabled=!count||over;
 status.classList.toggle('over',over);
 status.textContent=over?'Goal에는 Jira·Confluence 문서를 5개까지 넣을 수 있습니다(현재 '+count+'개). 하나를 지우면 미리볼 수 있습니다.':count?'참조 '+count+'/'+SOURCE_LIMIT:'';
}
document.getElementById('goal-entry').addEventListener('input',sourceLimit);
