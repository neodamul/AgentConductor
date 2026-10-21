'use strict';
const $ = id => document.getElementById(id);
let snapshot = null, selected = 0, review = null, polling = false, activeTab = 'activity', lastJob = null;
const drafts = new Map();let resumeAfter=null;
// DD-117: the coordinator row is selectable like a task row and shows its own work.
let coordinatorView=false;
let sessionStamp='', quotaAgent='',quotaActive='';const quotaRefreshed=new Map();
function statusLight(status){return ['running','completed','ready'].includes(status)?'green':['failed','blocked'].includes(status)?'red':'yellow';}
const names = {claude:'Claude Code',codex:'Codex',opencode:'OpenCode'};
const pretty = value => {const key='state.'+String(value||'unknown');return Object.hasOwn(AC_MESSAGES.en,key)?acMessage(key):String(value||acMessage('room.unknown')).replaceAll('_',' ');};
// Run and task states: a capitalised label and a colour tone, so the final state reads at a glance.
const STATUS_TONE = {completed:'ok',approved:'ok',merged:'ok',failed:'bad',rejected:'bad',running:'run',pending:'idle',ready:'idle'};
const statusTone = status => STATUS_TONE[status]||'wait';
// Literal labels so every state has an exact interface string to translate.
const STATUS_LABELS = {get completed(){return acMessage("startup.completed_22a970");},get failed(){return acMessage("room.failed_031a8f");},get pending(){return acMessage("room.pending");},get running(){return acMessage("room.running");},get ready(){return acMessage("startup.ready");},get blocked(){return acMessage("room.blocked");},get stopped(){return acMessage("room.stopped");},get approved(){return acMessage("room.approved");},get rejected(){return acMessage("room.rejected");},get waiting_approval(){return acMessage("room.waiting_approval");},get waiting_tool_approval(){return acMessage("room.waiting_tool_approval");},get waiting_availability(){return acMessage("room.waiting_availability");},get interrupted(){return acMessage("room.interrupted");}};
const statusLabel = status => STATUS_LABELS[status]||(text=>text.charAt(0).toUpperCase()+text.slice(1))(pretty(status));
function node(tag, text, cls) { const el=document.createElement(tag);if(text!==undefined)AC_I18N.label(el,text);if(cls)el.className=cls;return el; }
function notice(message,error=false) { AC_I18N.text($('notice'),message);$('notice').className='notice'+(error?' error':'');$('notice').hidden=false;clearTimeout(notice.timer);notice.timer=setTimeout(()=>{$('notice').hidden=true;},error?11000:5000); }
// DD-40: installed application icons for built-in agents; custom agents keep letters.
let iconCache={},iconKinds='';const ICONS=()=>iconCache;
function avatar(parser,label,cls){const url=parser&&parser!=='custom'?ICONS()[parser]:null;if(url){const img=document.createElement('img');img.src=url;img.alt='';img.className=cls+' app-icon';img.setAttribute('aria-hidden','true');return img;}const span=document.createElement('span');span.className=cls;AC_I18N.bind(span,()=>((label||'?').slice(0,1).toUpperCase()));return span;}
async function request(packet) {
  const bridge=window.webkit?.messageHandlers?.agentconductor??(window.__acRequest?{postMessage:window.__acRequest}:null);
  if(!bridge)throw Error(acMessage("startup.native_bridge_unavailable"));
  // A timed-out mutation remains accepted. Never automatically retry it.
  let timer;
  // Agent Room requests are marked so the workspace page can stay open beside the room.
  try {return JSON.parse(await Promise.race([bridge.postMessage(JSON.stringify({...packet,room:true})),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error(acMessage("startup.native_response_timed_out_refresh_before_retrying"))),15000);})]));}
  finally {clearTimeout(timer);}
}

function busy() { return snapshot?.job?.state==='running'; }
function idle() { return snapshot && !busy() && !['running','completed'].includes(snapshot.report.state); }
// A task that has not started can change agent or model while others run; the backend queues the change (#103).
function canEditTask(t) { return idle() || (snapshot?.report.state === 'running' && !busy() && t?.status === 'pending'); }
// A running task takes feedback that the backend queues for its next attempt (#95); otherwise the run must be idle.
function feedbackEnabled(task) { return !coordinatorView && !busy() && (snapshot.report.state === 'running' ? task?.status === 'running' : snapshot.report.state !== 'completed'); }
function task() { return snapshot?.report.tasks[selected]; }
// The watched session finished (#109): follow the run to the next running session, or to the integration entry when the run awaits a decision.
function followTarget(prev, next, selectedId, onCoordinator) { if (!prev || onCoordinator) return null; const was = prev.report.tasks.find(t => t.id === selectedId), now = next.report.tasks.find(t => t.id === selectedId); if (!was || was.status !== 'running' || !now || now.status === 'running') return null; const active = next.report.tasks.find(t => ['running', 'waiting_tool_approval'].includes(t.status)); if (active) return { id: active.id }; return ['waiting_approval', 'approved', 'rejected'].includes(next.report.state) ? { coordinator: true } : null; }
function followProgress(prev) { const target = followTarget(prev, snapshot, task()?.id, coordinatorView); if (!target) return; if (target.coordinator) selectCoordinator(); else selectTask(snapshot.report.tasks.findIndex(t => t.id === target.id)); }
function formatAge(ms) { const s = Math.max(0, Math.floor(ms / 1000)); if (s < 60) return s + 's'; const m = Math.floor(s / 60); if (m < 60) return m + 'm ' + String(s % 60).padStart(2, '0') + 's'; return Math.floor(m / 60) + 'h ' + String(m % 60).padStart(2, '0') + 'm'; }
// Facts only (#100): the tool in progress, the age of the last output and the process state; silence is never called thinking or failure.
function activityLines(a, extraMs = 0) { if (!a) return []; const lines = []; if (a.tool) lines.push(acMessage("room.activity_tool", { value1: a.tool.name, value2: formatAge((a.tool.running_ms ?? 0) + extraMs) })); lines.push(a.quiet_ms == null ? acMessage("room.activity_no_output") : acMessage("room.activity_last_output", { value1: formatAge(a.quiet_ms + extraMs) })); const state = { present: 'room.activity_process_present', absent: 'room.activity_process_absent', unknown: 'room.activity_process_unknown' }[a.process]; if (state) lines.push(acMessage(state)); return lines; }
let activitySeen = null, activityAt = 0;
function renderActivity() { const el = $('task-activity'); if (!el) return; const a = snapshot?.activity?.[task()?.id]; if (snapshot !== activitySeen) { activitySeen = snapshot; activityAt = Date.now(); } const lines = coordinatorView ? [] : activityLines(a, Date.now() - activityAt); el.hidden = !lines.length; el.textContent = lines.join(' · '); }
setInterval(() => { if (!document.hidden) renderActivity(); }, 1000);
function planTask() { return snapshot?.report.plan.tasks[selected]; }
function profile() { return snapshot?.report.plan.agents.find(a=>a.id===planTask()?.agent); }
function agentName(agent) {return names[agent?.parser]||agent?.id||acMessage("room.agent");}
// The plan review's one-line task summary: the first line that says what the task does.
function taskLine(prompt){const line=String(prompt||'').split('\n').map(l=>l.trim().replace(/^[-*]\s+/,'')).find(l=>l&&!/:$/.test(l))||'';return line.length>160?line.slice(0,160)+'…':line;}
// Task rows lead with the reviewed summary; the task ID stays visible as a small label.
function taskHeading(id,prompt){const heading=node('span',undefined,'session-task'),summary=taskLine(prompt);heading.append(node('span',id,'task-id'));if(summary)heading.append(node('span',summary,'task-summary'));AC_I18N.bind(heading,()=>(summary?id+' · '+summary:id),"title");return heading;}
function refreshQuota(agent){quotaActive=agent;quotaRefreshed.set(agent,Date.now());return act({kind:'quota',agent});}
function renderQuota(agent) {
 setOptions($('quota-agent'),snapshot.report.plan.agents.map(a=>({value:a.id,label:agentName(a)+' · '+a.id})),quotaAgent||agent?.id,JSON.stringify(snapshot.report.plan.agents.map(a=>a.id)));
 agent=snapshot.report.plan.agents.find(a=>a.id===$('quota-agent').value)||agent;
 const quota=snapshot.quotas?.[agent?.id];
 AC_I18N.bind($('quota-refresh'),()=>(acMessage("room.refresh_quota")));
 AC_I18N.bind($('quota-refresh'),()=>(agent?.parser==='claude'?acMessage("room.on_macos_query_claude_usage_without_generation_otherwise"):acMessage("room.request_account_metadata_from_the_cli")),"title");
 // Budgets (Enterprise) read as a remaining percentage, like the agent cards; amounts stay below.
 const budgetRows=(snapshot.quota_budget_rows?.[agent?.id]||[]).map(b=>{const row=node('p',undefined,'quota-window');row.append(node('strong',b.headline));if(b.amounts)row.append(node('span',b.amounts,'small muted'));if(b.reset_label)row.append(node('span',b.reset_label,'small muted'));return row;});
 const windows=quota?.windows||[],labels=snapshot.quota_window_labels?.[agent?.id]||[];
 const resetLabels=snapshot.quota_window_reset_labels?.[agent?.id]||[];
 AC_I18N.replace($('quota-windows'),...budgetRows,...(windows.length?windows.map((w,index)=>{const row=node('p',undefined,'quota-window');const label=labels[index]||w.label;row.append(node('strong',()=>(`${typeof w.remaining_percent==='number'?acMessage("room.remaining", {value1:Math.round(w.remaining_percent)}):acMessage("room.unavailable")} · ${label}`)));if(resetLabels[index])row.append(node('span',resetLabels[index],'small muted'));return row;}):budgetRows.length?[]:[node('p',()=>(quota?.error||acMessage("room.account_quota_not_queried")),'small muted')]));
 AC_I18N.bind($('quota-source'),()=>(quota?.observed_at?acMessage("room.observed_bcdb80", {value1:pretty(quota.status),value2:new Date(quota.observed_at*1000).toLocaleString(),value3:quota.source}):''));
 $('quota-refresh').disabled=busy()||snapshot.report.state==='running'||!agent;
}
function selectTask(index) {
  if(task()&&!coordinatorView)drafts.set(task().id,$('feedback').value);
  coordinatorView=false;selected=index;$('feedback').value=drafts.get(task()?.id)||'';renderSnapshot();loadTask();
}
function selectCoordinator() {
  if(task()&&!coordinatorView)drafts.set(task().id,$('feedback').value);
  coordinatorView=true;$('feedback').value='';renderSnapshot();loadTask();
}
function changeTab(tab) {
  activeTab=tab;for(const el of document.querySelectorAll('[data-tab]')){const active=el.dataset.tab===tab;el.classList.toggle('active',active);el.setAttribute('aria-selected',String(active));}
  for(const name of ['activity','review','logs','knowledge'])$(name+'-view').hidden=name!==tab;
}
function setOptions(select,values,current,stamp) {
  if(select.dataset.stamp===stamp)return;const chosen=document.activeElement===select?select.value:current;
  AC_I18N.replace(select,...values.map(value=>{const option=node('option',value.label);option.value=value.value;return option;}));
  if(values.some(v=>v.value===chosen))select.value=chosen;select.dataset.stamp=stamp;
}
function renderAvailability(t,a,quota,timer,error){
 const input=$('resume-at'),at=(t.restriction||quota?.restriction)?.available_at,owner=t.id+'|'+a?.id,binding=owner+'|'+(at??'');
 if(input.dataset.binding!==binding&&(input.dataset.owner!==owner||document.activeElement!==input)){const d=at?new Date(at*1000):null,pad=v=>String(v).padStart(2,'0');input.value=d?d.getFullYear()+'-'+pad(d.getMonth()+1)+'-'+pad(d.getDate())+'T'+pad(d.getHours())+':'+pad(d.getMinutes()):'';input.dataset.binding=binding;input.dataset.owner=owner;}
 const line=$('availability-timer');line.hidden=!timer&&!error;
 AC_I18N.bind(line,()=>{if(error)return acMessage('room.auto_resume_timer_paused',{reason:error});if(!timer)return '';const seconds=Math.max(0,Math.ceil(timer.at-Date.now()/1000)),pad=v=>String(v).padStart(2,'0'),remaining=pad(Math.floor(seconds/3600))+':'+pad(Math.floor(seconds%3600/60))+':'+pad(seconds%60);return acMessage(timer.kind==='fallback'?'room.auto_fallback_countdown':'room.auto_resume_countdown',{remaining,time:new Date(timer.at*1000).toLocaleString()});});
}
// availability-end
function renderSnapshot() {
  if(!snapshot)return;const r=snapshot.report;if(selected>=r.tasks.length)selected=0;
  const t=task(),p=planTask(),a=profile();
  let ownership=$('owned-file-warning');if(!ownership){ownership=node('p');ownership.id='owned-file-warning';$('run-banner-progress').parentElement.append(ownership);}const changed=snapshot.owned_changes||[];ownership.hidden=!r.direct_workspace||r.plan.workspace_mode!=='shared'||(!changed.length&&r.state!=='running');AC_I18N.bind(ownership,()=>changed.length?acMessage("room.owned_edit_detected",{paths:changed.join(', ')}):acMessage("room.owned_edit_warning",{paths:r.plan.tasks.flatMap(t=>t.paths).join(', ')}));
  AC_I18N.bind($('goal'),()=>(r.plan.goal));const project=(r.folder?.original||r.repo).replace(/^\\\\\?\\/,'');AC_I18N.bind($('repository'),()=>(project.split(/[\\/]/).filter(Boolean).pop()||project));AC_I18N.bind($('repository'),()=>(project),"title");
  AC_I18N.bind($('workspace-mode'),()=>(r.plan.workspace_mode==='worktree'?acMessage("room.isolated_worktrees"):acMessage("room.shared_directory")));AC_I18N.bind($('branch'),()=>(r.branch));
  AC_I18N.bind($('run-state'),()=>(statusLabel(r.state)));$('run-state').className='state-label tone-'+statusTone(r.state);AC_I18N.bind($('session-count'),()=>(r.tasks.length));
  const done=r.tasks.filter(t=>t.status==='completed').length,failure=r.error||r.tasks.find(t=>t.status==='failed'&&t.error)?.error||'';
  $('run-banner').className='run-banner tone-'+statusTone(r.state);AC_I18N.bind($('run-banner-state'),()=>(statusLabel(r.state)));
  const ph=snapshot.phase,phIndex=ph?(ph.phases||[]).findIndex(x=>x.id===ph.phase):-1;// DD-122: the titlebar names the run's phase and what it covers, beside the workspace.
  for(const id of ['start-run-phase','run-phase']){const box=document.getElementById(id);if(!box)continue;box.hidden=phIndex<0;AC_I18N.replace(box);if(phIndex>=0){box.append(node('strong',()=>(acMessage("room.phase", {value1:phIndex+1,value2:ph.phases.length,value3:ph.phases[phIndex].name||ph.phase}))),node('span',ph.description||'','run-phase-detail'));AC_I18N.bind(box,()=>(`${ph.slug||''} · ${ph.description||''}`),"title");}}
  AC_I18N.bind($('run-banner-progress'),()=>(acMessage("room.tasks_completed", {value1:done,value2:r.tasks.length})+(phIndex>=0?acMessage("room.phase_of", {value1:phIndex+1,value2:ph.phases.length,value3:ph.phases[phIndex].name||ph.phase}):'')));
  const broken=r.tests.find(x=>x.exit_code!==0||x.timed_out);const why=broken?acMessage("room.integration_verification_failed_for", {value1:broken.task,value2:clipLines(broken.output||'',4)}):failure?acMessage("room.last_error")+String(failure).slice(0,300):'';AC_I18N.bind($('run-banner-error'),()=>(why));$('run-banner-error').hidden=!why;$('banner-fix').hidden=!broken||!idle();$('banner-fix').onclick=()=>fixIntegration(broken);
  const final=finalStep(r),stamp=JSON.stringify([selected,coordinatorView,final,r.tasks.map((t,i)=>[t.id,t.status,r.plan.tasks[i].agent]),r.plan.agents.map(a=>[a.id,a.parser]),snapshot.quotas,snapshot.job,Object.keys(snapshot.icons||{})]);
  if(stamp!==sessionStamp){sessionStamp=stamp;AC_I18N.replace($('sessions'),...r.tasks.map((session,index)=>{
    const agent=r.plan.agents.find(a=>a.id===r.plan.tasks[index].agent);const row=node('div',undefined,'session attn-'+statusTone(session.status)+(session.status==='running'?' is-running':'')+(selected===index&&!coordinatorView?' active':'')),button=node('button',undefined,'session-choice');AC_I18N.bind(button,()=>(acMessage("room.select", {value1:agentName(agent),value2:session.id,value3:statusLabel(session.status)})),"aria-label");const light=node('span',undefined,'agent-light '+statusLight(session.status)+(session.status==='running'?' running':''));light.setAttribute('aria-hidden','true');button.append(light);
    const body=node('span',undefined,'session-body'),meta=node('span',undefined,'session-meta');// DD-137 (user, 2026-10-14): the task name leads; a small icon, the agent name and a compact
    // status tag share the next line, and the tag wraps below when the rail is too narrow.
    meta.append(avatar(agent?.parser,agentName(agent),'session-symbol'),node('span',agentName(agent),'name'),node('span',statusLabel(session.status),'sub session-status tone-'+statusTone(session.status)));body.append(taskHeading(session.id,r.plan.tasks[index]?.prompt),meta);button.append(body);button.onclick=()=>selectTask(index);row.prepend(button);return row;
  }),finalRow(final));}
  const stage=r.state==='completed'?'complete':['waiting_approval','approved','rejected'].includes(r.state)?'verify':r.tasks.some(t=>t.attempt>0)?'task':'plan';const stages=['plan','task','verify','complete'];for(const el of document.querySelectorAll('[data-stage]')){el.classList.toggle('current',el.dataset.stage===stage);el.classList.toggle('done',stages.indexOf(el.dataset.stage)<stages.indexOf(stage));}
  AC_I18N.bind($('selected-agent'),()=>(agentName(a)));$('agent-symbol').replaceWith(Object.assign(avatar(a?.parser,agentName(a),'agent-symbol'),{id:'agent-symbol'}));AC_I18N.bind($('selected-task'),()=>(t?.id||''));AC_I18N.bind($('feedback-target'),()=>(agentName(a)+' / '+(t?.id||'')));
  AC_I18N.bind($('task-status'),()=>(statusLabel(t?.status)));$('task-status').className='status-pill tone-'+statusTone(t?.status);
  AC_I18N.bind($('paths'),()=>(p?.paths.join('\n')||'—'));AC_I18N.bind($('session-id'),()=>(t?.session_id?acMessage("room.session_168693")+t.session_id:acMessage("room.no_native_session_yet")));
  setOptions($('agent-select'),r.plan.agents.map(a=>({value:a.id,label:agentName(a)+' · '+a.id})),p.agent,t.id+p.agent+JSON.stringify(r.plan.agents.map(a=>a.id)));
  const models=a?.catalog?.models||[];setOptions($('model-select'),models.length?models.map(m=>({value:m,label:m})):[{value:'',label:acMessage("room.no_observed_catalog")}],p.model||models[0]||'',t.id+JSON.stringify(a?.catalog)+String(p.model));
  AC_I18N.bind($('catalog-note'),()=>(a?.catalog?acMessage("room.observed", {value1:a.catalog.cli_version,value2:new Date(a.catalog.queried_at*1000).toLocaleDateString()}):acMessage("room.import_or_discover_a_model_catalog_with_the")));
  $('set-model').disabled=!canEditTask(t)||!models.length;$('reassign').disabled=!canEditTask(t);$('schedule').disabled=!idle()||t.status!=='waiting_availability';
  const effectiveRestriction=t.restriction||snapshot.quotas?.[a?.id]?.restriction;
  AC_I18N.bind($('restriction'),()=>(effectiveRestriction?(effectiveRestriction.available_at?acMessage("room.available", {value1:new Date(effectiveRestriction.available_at*1000).toLocaleString()}):acMessage("room.reset_time_unknown_choose_a_time_below"))+' · '+((effectiveRestriction.source==='quota_observation'||effectiveRestriction.reason==='room.account_quota_exhausted')?acMessage("room.account_quota_exhausted"):String(effectiveRestriction.reason||effectiveRestriction.message||t.error||'').slice(0,180)):acMessage("room.no_recorded_restriction")));
  renderAvailability(t,a,snapshot.quotas?.[a?.id],(snapshot.availability_timer||[]).find(entry=>entry.task===t.id&&entry.agent===a?.id),snapshot.availability_timer_error);
  if(coordinatorView){const c=r.plan.agents.find(x=>x.id===snapshot.coordinator)||null;AC_I18N.bind($('selected-agent'),()=>(acMessage("room.coordinator")));$('agent-symbol').replaceWith(Object.assign(node('span','◎','agent-symbol'),{id:'agent-symbol'}));AC_I18N.bind($('selected-task'),()=>(acMessage("room.integration_verification_tool_decisions_and_run_steps")));AC_I18N.bind($('feedback-target'),()=>(acMessage("room.coordinator")));AC_I18N.bind($('task-status'),()=>(statusLabel(final.status)));$('task-status').className='status-pill tone-'+statusTone(final.status);AC_I18N.bind($('paths'),()=>('—'));AC_I18N.bind($('session-id'),()=>(c?acMessage("room.agent_8739d3", {value1:c.id}):''));}
  renderActivity();
  $('stop').disabled=!['running','ready','waiting_availability','waiting_tool_approval','stopped'].includes(r.state);
  $('resume').disabled=!idle()||['completed','approved','waiting_approval','rejected'].includes(r.state);$('resume').className=$('resume').disabled?'quiet':'primary';$('banner-resume').hidden=$('resume').disabled;$('recover').disabled=busy()||r.state!=='running';
  // The button that started a UI job stays marked busy until that job ends.
  {const job=snapshot.job?.state==='running'?snapshot.job.kind:null;if(typeof AC_PRESS!=='undefined')for(const[id,kinds]of[['resume',['resume','auto_resume']],['approve',['approve']],['merge',['merge']],['recover',['recover']],['reject',['reject']]])AC_PRESS.hold($(id),kinds.includes(job));}
  $('send-feedback').disabled=!feedbackEnabled(t);AC_I18N.bind($('feedback'),()=>(coordinatorView?acMessage("room.select_a_task_to_direct_it_or_use"):acMessage("room.request_a_change_point_out_an_edge_case")),"placeholder");
  AC_I18N.bind($('job-state'),()=>(busy()&&snapshot.job.kind!=='quota'?pretty(snapshot.job.kind)+acMessage("room.in_progress"):snapshot.job?.state==='failed'&&snapshot.job.kind!=='quota'?acMessage("room.action_needs_attention"):''));
  AC_I18N.bind($('completion'),()=>(`${r.tasks.filter(t=>t.status==='completed').length} / ${r.tasks.length}`));
  AC_I18N.bind($('candidate-sha'),()=>(r.candidate?.commit||acMessage("room.no_candidate")));AC_I18N.bind($('test-count'),()=>(r.tests.length?acMessage("room.passed", {value1:r.tests.filter(t=>t.exit_code===0&&!t.timed_out).length}):'—'));
  const currentReview=review&&review.revision===snapshot.revision;
  $('review-stale').hidden=!review||review.revision===snapshot.revision;
  $('approve').disabled=!idle()||r.state!=='waiting_approval'||!currentReview;$('reject').disabled=!idle()||!r.candidate;
  $('merge').disabled=!idle()||r.state!=='approved'||!currentReview;
  AC_I18N.bind($('merge'),()=>(r.direct_workspace?acMessage("room.accept_changes"):r.folder?acMessage("room.apply_changes"):acMessage("room.merge")));
  AC_I18N.bind($('candidate-note'),()=>(r.direct_workspace?(r.state==='completed'?acMessage("room.accepted_complete"):r.state==='approved'?acMessage("room.approved_accept_changes"):acMessage("room.review_diff_and_tests_before_approval")):r.folder?(r.state==='completed'?acMessage("room.changes_applied_this_run_is_complete"):r.state==='approved'?acMessage("room.approved_applying_changes_remains_a_separate_action"):acMessage("room.review_diff_and_tests_before_approval")):r.state==='completed'?acMessage("room.merged_this_run_is_complete"):r.state==='approved'?acMessage("room.approved_merging_remains_a_separate_action"):review&&!currentReview?acMessage("room.state_changed_inspect_the_latest_evidence"):acMessage("room.review_diff_and_tests_before_approval")));
  nextStep(r,currentReview);renderQuota(a);renderTools();AC_I18N.bind($('run-path'),()=>(snapshot.run_dir));AC_I18N.bind($('connection'),()=>(acMessage("room.connected_local_run")));
}
// DD-112: provider events read like the CLI transcript; raw JSON stays in the Raw logs tab.
const clipLines=(text,max=8)=>{const all=String(text??'').replace(/\s+$/,'').split('\n');return all.length>max?all.slice(0,max).join('\n')+acMessage("room.more_lines", {value1:all.length-max}):all.join('\n');};
const resultText=c=>typeof c==='string'?c:Array.isArray(c)?c.map(x=>x?.text||'').join('\n'):c?.text||'';
function toolLine(name,input){const i=input&&typeof input==='object'?input:{};const arg=i.command??i.file_path??i.filePath??i.path??i.pattern??i.url??i.query??'';return `● ${name}(${String(arg).split('\n')[0].slice(0,200)})`+(i.description&&i.command?`\n  ${i.description}`:'');}
function transcript(e) {
  const raw=e.payload&&typeof e.payload==='object'?e.payload:{},out=[],add=(label,text,cls='')=>out.push({label,text:String(text??''),cls});
  const it=raw.item||{},done=raw.type==='item.completed';
  switch(raw.type){
    case 'system':add(acMessage("room.session"),raw.subtype==='init'?acMessage("startup.started")+(raw.model?' · '+raw.model:''):statusLabel(raw.subtype),'status');break;
    case 'assistant':for(const c of raw.message?.content||[]){if(c.type==='text'&&c.text?.trim())add(acMessage("room.agent"),c.text);else if(c.type==='tool_use')add(acMessage("room.tool"),toolLine(c.name,c.input),'tool');else if(c.type==='thinking'&&c.thinking)add(acMessage("room.thinking"),clipLines(c.thinking,4),'muted');}break;
    case 'user':for(const c of raw.message?.content||[])if(c.type==='tool_result')add(c.is_error?acMessage("room.tool_error"):acMessage("room.result"),clipLines(resultText(c.content))||acMessage("common.no_output"),c.is_error?'error':'result');break;
    case 'result':add(raw.is_error?acMessage("room.failed_031a8f"):acMessage("room.finished"),typeof raw.result==='string'?raw.result:statusLabel(raw.subtype),raw.is_error?'error':'status');break;
    case 'thread.started':add(acMessage("room.session"),acMessage("startup.started"),'status');break;
    case 'turn.completed':if(raw.usage)add(acMessage("room.usage"),acMessage("room.in_out", {value1:raw.usage.input_tokens??0,value2:raw.usage.output_tokens??0}),'muted');break;
    case 'turn.failed':case 'error':add(acMessage("room.error"),raw.error?.message||raw.message||acMessage("room.error"),'error');break;
    case 'item.started':case 'item.updated':case 'item.completed':
      if(it.type==='agent_message'&&done)add(acMessage("room.agent"),it.text);
      else if(it.type==='reasoning'&&done&&it.text)add(acMessage("room.thinking"),clipLines(it.text,4),'muted');
      else if(it.type==='command_execution'){if(raw.type==='item.started')add(acMessage("room.tool"),`● $ ${it.command}`,'tool');else if(done)add(it.exit_code===0?acMessage("room.result"):acMessage("room.tool_error"),acMessage("room.exit_e1fe73", {value1:it.exit_code??'?'})+(it.aggregated_output?'\n'+clipLines(it.aggregated_output):''),it.exit_code===0?'result':'error');}
      else if(it.type==='file_change'&&done)add(acMessage("startup.edit"),(it.changes||[]).map(c=>`● ${statusLabel(c.kind)} ${c.path}`).join('\n'),'tool');
      else if(it.type==='mcp_tool_call'&&done)add(acMessage("room.tool"),`● ${it.server||'mcp'}.${it.tool||''}`,'tool');
      else if(it.type==='web_search'&&done)add(acMessage("room.tool"),acMessage("room.search", {value1:it.query||''}),'tool');
      else if(it.type==='todo_list')add(acMessage("room.plan"),(it.items||[]).map(x=>(x.completed?'☑ ':'☐ ')+x.text).join('\n'),'muted');
      else if(it.type==='error')add(acMessage("room.error"),it.message,'error');
      break;
    case 'text':if(raw.part?.text)add(acMessage("room.agent"),raw.part.text);break;
    case 'tool_use':{const st=raw.part?.state||{};add(acMessage("room.tool"),toolLine(raw.part?.tool||'tool',st.input),'tool');if(st.output)add(acMessage("room.result"),clipLines(st.output),'result');break;}
    case 'step_start':case 'step_finish':case 'turn.started':case 'rate_limit_event':break;
    default:{const text=raw.payload?.summary??raw.summary??raw.text??raw.message;add(statusLabel(raw.type||e.kind),typeof text==='string'?text:'',e.kind==='error'?'error':'muted');}
  }
  return out;
}
// DD-113: the coordinator's final step (integration verification) is listed after the tasks.
function finalStep(r) {
  const failed=r.tests.filter(t=>t.exit_code!==0||t.timed_out),allDone=r.tasks.every(t=>t.status==='completed');
  const status=failed.length?'failed':r.tests.length&&allDone?'completed':allDone&&r.state==='running'?'running':'pending';
  return {status,detail:failed.length?acMessage("room.failed", {value1:failed.map(t=>t.task).join(', ')}):r.tests.length?acMessage("room.task_tests_passed", {value1:r.tests.length}):acMessage("room.runs_every_task_test_on_the_combined_result_96053d")};
}
function finalRow(final) {
  const row=node('div',undefined,'session final-step attn-'+statusTone(final.status)+(coordinatorView?' active':'')),button=node('button',undefined,'session-choice');
  AC_I18N.bind(button,()=>(acMessage("room.coordinator_integration_verification", {value1:statusLabel(final.status)})),"aria-label");
  const symbol=node('span','◎','session-symbol');symbol.setAttribute('aria-hidden','true');
  const body=node('span',undefined,'session-body'),meta=node('span',undefined,'session-meta');meta.append(symbol,node('span',()=>(acMessage("room.coordinator")),'name'),node('span',statusLabel(final.status),'sub session-status tone-'+statusTone(final.status)));
  body.append(node('span',()=>(acMessage("room.integration_verification")),'session-task'),meta,node('span',final.detail,'sub task-name'));
  button.append(body);button.onclick=selectCoordinator;row.append(button);return row;
}
// Selects the task whose test failed on the combined result and drafts the direction to send.
function fixIntegration(test) {
  if(!test)return;const index=snapshot.report.tasks.findIndex(t=>t.id===test.task);if(index<0)return;
  selectTask(index);
  $('feedback').value=acMessage("room.the_coordinator_s_integration_verification_ran_this_task", {value1:clipLines(test.output||acMessage("common.no_output"),12)});
  $('feedback').focus();notice(()=>(acMessage("room.review_the_drafted_direction_then_send_feedback_the")));
}
function renderCoordinator(data) {
  if(!coordinatorView)return;const r=snapshot.report;
  const timeline=[event(acMessage("room.coordinator"),acMessage("room.runs_every_task_test_on_the_combined_result"),'plan')];
  for(const h of data.history||[]){if(h.event==='tool_orchestrator_decision'){timeline.push(decisionEntry(h.payload));continue;}const bad=/fail|error|reject|interrupt/i.test(h.event+' '+h.summary);timeline.push(event(statusLabel(h.event),h.summary||'',bad?'error':'status',h.seq));}
  for(const t of r.tests){const ok=t.exit_code===0&&!t.timed_out;timeline.push(event(ok?acMessage("room.integration_test_passed"):acMessage("room.integration_test_failed"),acMessage("room.exit_df653f", {value1:t.task,value2:t.exit_code??'?',value3:t.timed_out?acMessage("common.timed_out"):'',value4:t.output?'\n'+clipLines(t.output,12):''}),ok?'result':'error'));}
  if(r.error)timeline.push(event(acMessage("room.run_error"),r.error,'error'));
  const activity=$('activity'),nearBottom=activity.parentElement.scrollHeight-activity.parentElement.scrollTop-activity.parentElement.clientHeight<70;
  AC_I18N.replace(activity,...timeline);if(nearBottom)activity.parentElement.scrollTop=activity.parentElement.scrollHeight;
  AC_I18N.bind($('stdout'),()=>(data.stdout||acMessage("room.no_verification_output_yet")));AC_I18N.bind($('stderr'),()=>(data.stderr||acMessage("room.no_errors_logged")));
  AC_I18N.replace($('lessons'),node('p',()=>(acMessage("room.lessons_belong_to_tasks_select_a_task_to")),'empty'));
}
// DD-119: the banner always says what to do next and offers that action.
function completedStep(home) {
  const ph=snapshot.phase,phases=ph?.phases||[],i=phases.findIndex(x=>x.id===ph?.phase),n=phases.findIndex(x=>x.id===ph?.current_phase);
  if(i<0)return home?acMessage("room.this_run_is_complete_return_to_the_workspace"):acMessage("room.this_run_is_complete");
  if(n<0)return acMessage("room.phase_of_is_complete_and_with_it_every", {value1:i+1,value2:phases.length});
  return acMessage("room.phase_of_is_complete_phase_is_next_go", {value1:i+1,value2:phases.length,value3:phases[i].name||ph.phase,value4:n+1,value5:phases[n].name||ph.current_phase,value6:n+1});
}
function nextStep(r,currentReview) {
  const home=document.getElementById('startup-home');
  const steps={
    waiting_approval:currentReview?[acMessage("room.tests_passed_on_the_combined_result_check_the"),acMessage("room.approve"),()=>$('approve').click()]:[acMessage("room.all_tasks_and_integration_verification_passed_review_the"),acMessage("room.review_changes"),()=>loadReview()],
    approved:[r.direct_workspace?acMessage("room.approved_accept_changes"):r.folder?acMessage("room.approved_apply_the_changes_to_the_project_folder"):acMessage("room.approved_merge_the_candidate_to_finish_this_run"),r.direct_workspace?acMessage("room.accept_changes"):r.folder?acMessage("room.apply_changes"):acMessage("room.merge"),()=>{if(currentReview)$('merge').click();else loadReview();}],
    completed:[completedStep(home),home?acMessage("room.back_to_workspace"):'',()=>home?.click()],
    running:[acMessage("room.agents_are_working_decisions_they_need_appear_as"),'',null],
  };
  const [text,label,run]=steps[r.state]||['','',null];
  AC_I18N.bind($('run-banner-next'),()=>(text?acMessage("room.next")+text:''));$('run-banner-next').hidden=!text;
  const button=$('banner-next');AC_I18N.bind(button,()=>(label));button.hidden=!label||!run||(r.state!=='completed'&&!idle());button.onclick=run;
}
function decisionEntry(d) {
  let input={};try{input=JSON.parse(d.input);}catch{}
  const label=d.allow===true?acMessage("room.coordinator_allowed"):d.allow===false?acMessage("room.coordinator_denied"):acMessage("room.coordinator_could_not_decide");
  const when=d.decided_ms?new Date(d.decided_ms).toLocaleTimeString()+' · ':'';
  return event(label,`${when}${toolLine(d.tool,input)}\n${d.reason||d.error||''}`,d.allow===true?'status':'warning');
}
function event(label,text,cls='',seq) {
  const el=node('article',undefined,'event '+cls);el.append(node('span',undefined,'event-marker'));const body=node('div',undefined,'event-body');const head=node('div',undefined,'event-head');head.append(node('span',label));if(seq!==undefined)head.append(node('span','#'+seq,'event-seq'));body.append(head,node('p',text,'event-text'));el.append(body);return el;
}
function renderTask(data) {
  const t=task(),p=planTask();if(!t||data.task.id!==t.id)return;
  const timeline=[event(acMessage("room.assigned_task"),p.prompt,'plan')];if(t.feedback)timeline.push(event(acMessage("room.you_latest_review"),t.feedback));
  for(const e of data.events)for(const x of transcript(e))timeline.push(event(x.label,x.text,x.cls,e.seq));
  for(const d of data.decisions||[])timeline.push(decisionEntry(d));
  if(t.error)timeline.push(event(acMessage("room.coordinator_report"),t.error,'error'));
  if(!data.events.length)timeline.push(node('p',()=>(acMessage("room.no_provider_events_have_been_reported_yet")),'empty'));
  const activity=$('activity'),nearBottom=activity.parentElement.scrollHeight-activity.parentElement.scrollTop-activity.parentElement.clientHeight<70;
  AC_I18N.replace(activity,...timeline);if(nearBottom)activity.parentElement.scrollTop=activity.parentElement.scrollHeight;mirrorToolLog(t.id,timeline);
  AC_I18N.bind($('stdout'),()=>(data.stdout||acMessage("room.no_output_yet")));AC_I18N.bind($('stderr'),()=>(data.stderr||acMessage("room.no_errors_logged")));
  const notes=data.knowledge.filter(k=>k.task===t.id);AC_I18N.replace($('lessons'),...(notes.length?notes.map(k=>{const el=node('article',undefined,'lesson');el.append(node('h3',k.lesson.context),node('span',()=>(acMessage("room.confidence", {value1:pretty(k.status),value2:k.lesson.confidence})),'status-pill'),node('p',k.lesson.response),node('pre',k.lesson.sources.map(s=>s.path+'\n'+s.sha256).join('\n')),node('p',()=>(acMessage("room.revalidate")+k.lesson.revalidate),'small muted'));return el;}):[node('p',()=>(acMessage("room.no_reviewed_lessons_for_this_task_draft_observations")),'empty')]));
  const u=data.usage;AC_I18N.bind($('tokens'),()=>(u&&typeof u.input_tokens==='number'?acMessage("room.in_out", {value1:u.input_tokens.toLocaleString(),value2:(u.output_tokens||0).toLocaleString()}):acMessage("room.not_reported")));
  const duration=t.started_ms?(Number(t.finished_ms||Date.now())-Number(t.started_ms))/1000:null;AC_I18N.bind($('duration'),()=>(duration===null?'—':`${Math.max(0,duration).toFixed(1)}s`));
}
async function loadTask() {if(!snapshot||roomStopped)return;if(coordinatorView){try{const data=await request({kind:'coordinator'});if(!roomStopped)renderCoordinator(data);}catch(e){notice(e.message,true);}return;}const index=selected;try{const data=await request({kind:'task',index});if(!roomStopped&&index===selected)renderTask(data);}catch(e){notice(e.message,true);}}
async function act(action,revision=snapshot?.revision) {
  if(roomStopped)return false;
  try{const pending=request({kind:'action',revision,action}),accepted=await (typeof AC_PRESS!=='undefined'?AC_PRESS.track(pending):pending);if(action.kind!=='quota')notice(()=>(accepted.message||acMessage("room.accepted_waiting_for_the_coordinator", {value1:pretty(accepted.kind)})));await poll();return true;}catch(e){if(action.kind!=='quota')notice(e.message,true);await poll();return false;}
}
async function decision(title,description,evidence,needsReason=false,confirm=acMessage("room.confirm")) {
  const dialog=$('decision-dialog');AC_I18N.bind($('dialog-title'),()=>(title));AC_I18N.bind($('dialog-description'),()=>(description));AC_I18N.bind($('dialog-evidence'),()=>(evidence));
  $('dialog-reason').value='';$('dialog-reason').hidden=!needsReason;$('dialog-reason-label').hidden=!needsReason;$('dialog-reason').required=needsReason;AC_I18N.bind($('dialog-confirm'),()=>(confirm));dialog.returnValue='';dialog.showModal();
  return new Promise(resolve=>dialog.addEventListener('close',()=>resolve(dialog.returnValue==='confirm'?(needsReason?$('dialog-reason').value:true):null),{once:true}));
}
async function briefTeam(){
 try{const b=await request({kind:'briefing'});AC_I18N.bind($('briefing-time'),()=>(acMessage("room.captured_read_only", {value1:pretty(b.state),value2:new Date(b.observed_at_ms).toLocaleString()})));
 AC_I18N.replace($('briefing-sessions'),...b.sessions.map(s=>{const card=node('article',undefined,'briefing-session');card.append(node('h3',`${s.agent} · ${s.task} · ${pretty(s.state)}`),node('p',s.work),node('p',()=>(acMessage("room.session_model_attempt", {value1:s.session_id||acMessage("room.not_established"),value2:s.model||acMessage("startup.cli_default"),value3:s.attempt})),'small muted'),node('p',()=>(acMessage("room.process_observed", {value1:pretty(s.process.state),value2:new Date(s.process.observed_at_ms).toLocaleTimeString()})),'small muted'));
 if(s.last_activity)card.append(node('p',()=>(acMessage("room.last_activity", {value1:pretty(s.last_activity.kind),value2:pretty(s.last_activity.freshness),value3:s.last_activity.observed_at_ms?' · '+new Date(s.last_activity.observed_at_ms).toLocaleTimeString():''})),'small muted'),node('p',s.last_activity.text,'briefing-text'));
 const memory=s.memory_report;card.append(node('p',()=>(memory.summary?acMessage("room.memory_report", {value1:memory.source,value2:memory.freshness,value3:memory.summary}):memory.note),'small muted'));
 if(s.restriction)card.append(node('p',()=>(s.restriction.available_at?acMessage("room.available", {value1:new Date(s.restriction.available_at*1000).toLocaleString()}):acMessage("room.availability_reset_unknown")),'warning'));
 if(s.attention)card.append(node('p',s.attention,'warning'));card.append(node('p',()=>(acMessage("room.next")+s.next_action)));return card;}));
 AC_I18N.bind($('briefing-approvals'),()=>(b.approvals.length?acMessage("room.review_requests")+b.approvals.map(a=>`${a.kind} · ${a.task||acMessage("room.integration_candidate")} · ${a.status}`).join('; '):acMessage("room.no_recorded_approval_requests")));
 $('briefing-dialog').showModal();}catch(e){notice(e.message,true);}
}
async function loadReview() {
  try{const next=await request({kind:'evidence'});if(roomStopped)return;review=next;AC_I18N.bind($('review-caption'),()=>(review.report.candidate?acMessage("room.candidate_exact_captured_evidence", {value1:review.report.candidate.commit.slice(0,12)}):acMessage("room.no_candidate_is_available")));
    AC_I18N.replace($('test-results'),...review.test_output.map(t=>{const el=node('article',undefined,'test-result'+(t.exit_code!==0||t.timed_out?' failed':''));el.append(node('strong',()=>(acMessage("room.exit", {value1:t.exit_code===0&&!t.timed_out?'✓':'×',value2:t.task,value3:t.exit_code??acMessage("room.unknown"),value4:t.timed_out?acMessage("common.timeout"):''}))));if(t.stdout||t.stderr)el.append(node('pre',t.stdout+'\n'+t.stderr));return el;}));
    AC_I18N.replace($('diff'),...review.diff.split('\n').map(line=>node('span',line||' ','diff-line'+(line.startsWith('+')?' add':line.startsWith('-')?' remove':line.startsWith('@@')?' hunk':''))));
    if(review.truncated){$('diff').append(node('span','…','diff-line muted'),node('span',()=>acMessage("room.full_diff_cli",{command:'agentconductor diff "'+snapshot.run_dir+'"'}),'diff-line muted'));notice(()=>acMessage("room.diff_exceeds_the_inline_review_limit_use_the"));}
    changeTab('review');renderSnapshot();
  }catch(e){notice(e.message,true);}
}
function renderTools() {
  const requests=snapshot.tool_requests||[];const pending=requests.filter(r=>['pending','approved'].includes(r.status));AC_I18N.bind($('tool-count'),()=>(pending.length));$('tool-count').classList.toggle('alert',pending.length>0);AC_I18N.bind($('request-count'),()=>(pending.length));
  AC_I18N.replace($('tool-requests'),...(requests.length?requests.map(r=>{
    const el=node('article',undefined,'tool-request');el.append(node('strong',`${r.request.tool} · ${pretty(r.status)}`),node('p',r.context.task,'small muted'),toolView(r.request),node('code',r.id,'sha'));
    const actions=node('div',undefined,'tool-actions');for(const [kind,label] of [['tool_approve',acMessage("room.review_allow")],['tool_reject',acMessage("room.deny")],['tool_refresh',acMessage("room.refresh")]]) {
      // A running worker can be paused on a pending request; deciding it is the only live action.
      const live=snapshot.report.state==='running'&&kind!=='tool_refresh';
      const button=node('button',label);button.disabled=!(idle()||live)||(kind==='tool_refresh'?!['pending','approved'].includes(r.status):r.status!=='pending');
      button.onclick=async()=>{const revision=snapshot.revision;const ok=await decision(label+acMessage("room.tool_request_suffix"),kind==='tool_approve'?acMessage("room.authorize_the_exact_current_input_once_for_300"):kind==='tool_refresh'?acMessage("room.revoke_the_old_request_and_create_a_fresh"):acMessage("room.deny_this_exact_request_rework_or_reassignment_is"),JSON.stringify(r,null,2),false,label);if(ok)await act({kind,sha:r.id},revision);};actions.append(button);
    }el.append(actions);return el;
  }):[node('span',()=>(acMessage("room.no_recorded_requests")),'small muted')]));
  syncToolDialog();
}
// Tool requests in plain terms: what the agent wants to do and the inputs that matter.
// Unknown tools list every input; the exact JSON stays available under Raw input.
function toolSummary(tool,input) {
  const i=input||{},fields=[],add=(label,value,code)=>{if(value!==undefined&&value!==null&&value!=='')fields.push({label,value:typeof value==='string'?value:JSON.stringify(value,null,2),code:!!code});};let action;
  switch(tool){
    case 'Bash':case 'PowerShell':action=acMessage("room.run_a_command");add(acMessage("room.purpose"),i.description);add(acMessage("room.command"),i.command,true);if(i.run_in_background)add(acMessage("room.mode"),acMessage("room.in_the_background"));if(i.timeout)add(acMessage("room.time_limit"),`${Math.round(i.timeout/1000)} s`);break;
    case 'Read':action=acMessage("room.read_a_file");add(acMessage("room.file"),i.file_path,true);if(i.offset||i.limit)add(acMessage("room.lines"),`${i.offset||1}–${i.limit?(i.offset||1)+i.limit-1:acMessage("room.end")}`);break;
    case 'Write':action=acMessage("room.create_or_overwrite_a_file");add(acMessage("room.file"),i.file_path,true);add(acMessage("room.content"),i.content,true);break;
    case 'Edit':action=acMessage("room.edit_a_file");add(acMessage("room.file"),i.file_path,true);add(acMessage("room.replace"),i.old_string,true);add(acMessage("room.with"),i.new_string,true);if(i.replace_all)add(acMessage("room.scope"),acMessage("room.every_occurrence"));break;
    case 'MultiEdit':action=acMessage("room.edit_a_file");add(acMessage("room.file"),i.file_path,true);(i.edits||[]).forEach(e=>{add(acMessage("room.replace"),e.old_string,true);add(acMessage("room.with"),e.new_string,true);});break;
    case 'NotebookEdit':action=acMessage("room.edit_a_notebook");add(acMessage("room.file"),i.notebook_path,true);add(acMessage("room.cell"),i.cell_id);add(acMessage("room.content"),i.new_source,true);break;
    case 'Glob':action=acMessage("room.find_files");add(acMessage("room.pattern"),i.pattern,true);add(acMessage("room.in"),i.path,true);break;
    case 'Grep':action=acMessage("room.search_file_contents");add(acMessage("room.pattern"),i.pattern,true);add(acMessage("room.in"),i.path,true);add(acMessage("room.files"),i.glob||i.type,true);break;
    case 'WebFetch':action=acMessage("room.fetch_a_web_page");add(acMessage("room.url"),i.url,true);add(acMessage("room.question"),i.prompt);break;
    case 'WebSearch':action=acMessage("room.search_the_web");add(acMessage("room.query"),i.query);break;
    case 'Task':case 'Agent':action=acMessage("room.start_a_sub_agent");add(acMessage("room.purpose"),i.description);add(acMessage("room.instructions"),i.prompt,true);break;
    default:action=acMessage("room.use_a_tool");for(const [key,value] of Object.entries(i))add(key,value,true);
  }
  return {action,fields};
}
function toolView(request) {
  const summary=toolSummary(request.tool,request.input),view=node('div',undefined,'tool-view'),list=node('dl');
  for(const f of summary.fields){const value=node('dd');value.append(f.code?node('pre',f.value):node('span',f.value));list.append(node('dt',f.label),value);}
  const raw=node('details');raw.append(node('summary',()=>(acMessage("room.raw_input"))),node('pre',JSON.stringify(request.input,null,2)));
  view.append(node('strong',summary.action,'tool-action'),list,raw);return view;
}
// A live tool request opens a decision window with the task's activity log. Deciding
// later, approving or letting it expire leaves the log updating until it is closed.
let toolDialog=null;const deferredTools=new Set();
function syncToolDialog() {
  const dialog=$('tool-dialog'),requests=snapshot.tool_requests||[];
  const waiting=requests.find(r=>r.status==='pending'&&!deferredTools.has(r.id));
  if(!dialog.open&&waiting){
    toolDialog={id:waiting.id,task:waiting.context.task,receipt:waiting};AC_I18N.replace($('tool-dialog-log'));
    const index=snapshot.report.tasks.findIndex(t=>t.id===waiting.context.task);if(index>=0&&index!==selected)selectTask(index);else loadTask();
    dialog.showModal();
  }
  if(!dialog.open||!toolDialog)return;
  const current=requests.find(r=>r.id===toolDialog.id),pending=current?.status==='pending',shown=current||toolDialog.receipt;toolDialog.receipt=shown;
  AC_I18N.bind($('tool-dialog-title'),()=>(toolSummary(shown.request.tool,shown.request.input).action));
  AC_I18N.bind($('tool-dialog-reason'),()=>(`${shown.request.tool} · ${shown.context.task}`));
  const stamp=JSON.stringify(shown.request);if($('tool-dialog-input').dataset.stamp!==stamp){$('tool-dialog-input').dataset.stamp=stamp;AC_I18N.replace($('tool-dialog-input'),toolView(shown.request));}
  const left=Math.max(0,Math.round(shown.created_at+(snapshot.tool_decision_wait_secs||600)-Date.now()/1000));
  const status=$('tool-dialog-status');status.className='small'+(pending?'':' settled');
  AC_I18N.bind(status,()=>(pending?acMessage("room.waiting_for_your_decision_time_left", {value1:Math.floor(left/60),value2:String(left%60).padStart(2,'0')}):acMessage("room.no_longer_waiting", {value1:pretty(current?.status||'closed')})));
  // The paused worker belongs to the running job, so deciding is the live action that
  // must stay available while that job runs (same rule as the Tool requests list).
  const live=snapshot.report.state==='running';
  for(const id of ['tool-dialog-allow','tool-dialog-deny'])$(id).disabled=!pending||!(idle()||live)||toolDialog.deciding;AC_I18N.bind($('tool-dialog-later'),()=>(pending?acMessage("room.decide_later"):acMessage("room.close")));
}
function mirrorToolLog(id,timeline) {
  if(!$('tool-dialog').open||toolDialog?.task!==id)return;
  const log=$('tool-dialog-log'),nearBottom=log.scrollHeight-log.scrollTop-log.clientHeight<70||!log.childElementCount;
  AC_I18N.replace(log,...timeline.map(n=>n.cloneNode(true)));if(nearBottom)log.scrollTop=log.scrollHeight;
}
async function decideTool(kind) {if(!toolDialog)return;toolDialog.deciding=true;for(const id of ['tool-dialog-allow','tool-dialog-deny'])$(id).disabled=true;const idleRun=snapshot.report.state!=='running';try{if(await act({kind,sha:toolDialog.id})&&kind==='tool_approve'&&idleRun)resumeAfter='tool_approve';}finally{if(toolDialog)toolDialog.deciding=false;}syncToolDialog();}
$('tool-dialog-allow').onclick=()=>decideTool('tool_approve');$('tool-dialog-deny').onclick=()=>decideTool('tool_reject');
$('tool-dialog-later').onclick=()=>$('tool-dialog').close();
$('tool-dialog').onclose=()=>{if(toolDialog)deferredTools.add(toolDialog.id);toolDialog=null;};
function autoQuotaRefresh(){if(roomStopped||!snapshot||busy()||snapshot.report.state==='running')return;const agent=snapshot.report.plan.agents.find(a=>['claude','codex'].includes(a.parser)&&snapshot.quota_refresh_due?.[a.id]);if(agent)refreshQuota(agent.id);}
async function poll() {
  if(polling||roomStopped)return;polling=true;
  try{const kinds=JSON.stringify([...new Set((snapshot?.report?.plan?.agents||[]).map(a=>a.parser||'custom'))].sort());const next=await request({kind:'snapshot',quota_refresh_last:Object.fromEntries(quotaRefreshed),icons:!snapshot||kinds!==iconKinds});if(roomStopped)return;if(next.icons){iconCache=next.icons;iconKinds=JSON.stringify([...new Set((next.report?.plan?.agents||[]).map(a=>a.parser||'custom'))].sort());}const before=snapshot;snapshot=next;followProgress(before);const job=snapshot.job;
    if(job&&job.state!=='running'&&lastJob!==job.id&&job.kind!=='quota'){lastJob=job.id;
      // Feedback or an approval sent to a run that is not running continues it.
      if(resumeAfter&&job.kind===resumeAfter){resumeAfter=null;if(job.state!=='failed'&&['ready','stopped'].includes(snapshot.report.state))setTimeout(()=>act({kind:'resume'}),0);}if(job.state==='failed')notice(()=>(job.error||(job.termination_signal?acMessage("room.action_terminated_by_signal_previous_evidence_is_retained", {value1:job.termination_signal}):acMessage("room.action_failed_previous_evidence_is_retained"))),true);else{
      notice(()=>(acMessage("room.finished_run", {value1:pretty(job.kind),value2:pretty(snapshot.report.state)})));
      if(job.kind==='approve'&&review&&review.report.candidate?.commit===snapshot.report.candidate?.commit&&review.report.diff_hash===snapshot.report.diff_hash&&JSON.stringify(review.report.tests)===JSON.stringify(snapshot.report.tests))review.revision=snapshot.revision;
      for(const t of snapshot.report.tasks)if(t.feedback&&drafts.get(t.id)===t.feedback){drafts.delete(t.id);if(task()?.id===t.id&&!composing&&$('feedback').value===t.feedback)$('feedback').value='';}
    }}renderSnapshot();setTimeout(autoQuotaRefresh,0);await loadTask();
  }catch(e){if(roomStopped)return;AC_I18N.bind($('connection'),()=>(acMessage("room.local_connection_unavailable")));notice(e.message,true);}finally{polling=false;}
}
for(const el of document.querySelectorAll('[data-tab]'))el.onclick=()=>el.dataset.tab==='review'?loadReview():changeTab(el.dataset.tab);
$('brief-team').onclick=briefTeam;$('quota-agent').onchange=()=>{quotaAgent=$('quota-agent').value;renderQuota(profile());};$('quota-refresh').onclick=()=>refreshQuota($('quota-agent').value);$('refresh').onclick=poll;$('review-latest').onclick=loadReview;$('open-review').onclick=loadReview;$('open-tools').onclick=()=>{deferredTools.clear();if(snapshot)syncToolDialog();if(!$('tool-dialog').open)$('tool-requests').scrollIntoView({behavior:'smooth',block:'center'});};
// Input-method composition (e.g. Korean) must finish before text is replaced or sent.
let composing=false;document.addEventListener('compositionstart',()=>{composing=true;});document.addEventListener('compositionend',()=>{composing=false;});
$('feedback').oninput=()=>{if(task())drafts.set(task().id,$('feedback').value);};
$('send-feedback').onclick=async()=>{const feedback=$('feedback').value.trim();if(!feedback){notice(()=>(acMessage("room.enter_a_targeted_revision_before_sending")),true);return;}const idleRun=snapshot.report.state!=='running';if(await act({kind:'revise',task:task().id,feedback})&&idleRun)resumeAfter='revise';};
$('reassign').onclick=()=>act({kind:'reassign',task:task().id,agent:$('agent-select').value});$('set-model').onclick=()=>act({kind:'model',task:task().id,model:$('model-select').value});
$('schedule').onclick=()=>{const date=new Date($('resume-at').value);if(!Number.isFinite(date.getTime())){notice(()=>(acMessage("room.choose_a_valid_local_date_and_time")),true);return;}act({kind:'schedule',task:task().id,at:date.toISOString().replace('.000Z','Z')});};
$('resume').onclick=$('banner-resume').onclick=()=>act({kind:'resume'});$('recover').onclick=async()=>{const revision=snapshot.revision;if(await decision(acMessage("room.recover_interrupted_run"),acMessage("room.recovery_refuses_any_live_or_unknown_recorded_process"),snapshot.run_dir))await act({kind:'recover'},revision);};
$('stop').onclick=async()=>{const reason=await decision(acMessage("room.stop_the_run"),acMessage("room.stop_all_active_tasks_and_tests_retain_their"),snapshot.report.plan.goal,true,acMessage("room.stop_run"));if(reason)await act({kind:'stop',reason});};
$('approve').onclick=async()=>{const packet=review;if(!packet||packet.revision!==snapshot.revision)return;if(await decision(acMessage("room.approve_verified_candidate"),acMessage("room.approve_exactly_the_diff_and_test_evidence_shown"),acMessage("room.candidate_diff_tests_verified", {value1:packet.report.candidate.commit,value2:packet.report.diff_hash,value3:packet.report.tests.length}),false,acMessage("room.approve")))await act({kind:'approve',sha:packet.report.candidate.commit},packet.revision);};
$('merge').onclick=async()=>{const packet=review;if(!packet)return;if(await decision(packet.report.direct_workspace?acMessage("room.accept_changes"):packet.report.folder?acMessage("room.apply_approved_changes"):acMessage("room.merge_approved_candidate"),packet.report.direct_workspace?acMessage("room.accept_existing_files"):packet.report.folder?acMessage("room.apply_the_exact_verified_changes_to_the_original"):acMessage("room.integrate_this_exact_approved_candidate_into_the_target"),acMessage("room.candidate", {value1:packet.report.folder?acMessage("room.folder")+packet.report.folder.original:acMessage("room.branch")+packet.report.branch,value2:packet.report.candidate.commit}),false,packet.report.direct_workspace?acMessage("room.accept_changes"):packet.report.folder?acMessage("room.apply_changes"):acMessage("room.merge")))await act({kind:'merge',sha:packet.report.candidate.commit},packet.revision);};
$('reject').onclick=async()=>{const revision=snapshot.revision;const reason=await decision(acMessage("room.reject_candidate"),snapshot.report.direct_workspace?acMessage("room.reject_restores_baseline"):acMessage("room.reject_the_current_integration_candidate_send_targeted_feedback"),snapshot.report.candidate?.commit||acMessage("room.no_candidate"),true,acMessage("room.reject"));if(reason)await act({kind:'reject',reason},revision);};
const roomShortcut=e=>{if((e.metaKey||e.ctrlKey)&&e.key==='Enter'&&!e.isComposing&&!composing&&!$('decision-dialog').open){e.preventDefault();if(!$('send-feedback').disabled)$('send-feedback').click();}};
document.addEventListener('keydown',roomShortcut);
// DD-71: 1 s while the run or an action is active, 5 s when idle, paused while hidden.
let roomTimer=null,roomStopped=false;function roomSchedule(){clearTimeout(roomTimer);if(roomStopped||document.hidden)return;const fast=snapshot?.job?.state==='running'||['running','ready','waiting_availability'].includes(snapshot?.report?.state);roomTimer=setTimeout(async()=>{await poll();roomSchedule();},fast?1000:5000);}
const roomVisibility=()=>{if(document.hidden)clearTimeout(roomTimer);else if(!roomStopped)poll().then(roomSchedule);};document.addEventListener('visibilitychange',roomVisibility);
const roomLanguage=()=>{if(!roomStopped&&snapshot){sessionStamp='';renderSnapshot();loadTask();if(review&&activeTab==='review')loadReview();}};document.addEventListener('ac-language-change',roomLanguage);
poll().then(roomSchedule);window.stopRoom=()=>{roomStopped=true;clearTimeout(roomTimer);document.removeEventListener('visibilitychange',roomVisibility);document.removeEventListener('keydown',roomShortcut);document.removeEventListener('ac-language-change',roomLanguage);};
