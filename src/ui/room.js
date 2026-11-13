'use strict';
const $ = id => document.getElementById(id);
function resetReviewEvidence(){for(const id of ['review-summary','verify-rows','review-stale'])$(id).hidden=true;$('review-summary').replaceChildren();$('verify-rows').replaceChildren();$('test-results').replaceChildren();$('diff').replaceChildren(node('span',()=>acMessage('room.no_candidate_has_been_loaded')));$('diff').classList.add('empty');AC_I18N.bind($('review-caption'),()=>(acMessage('room.load_a_candidate_to_review_its_exact_evidence')));$('approve').disabled=true;$('merge').disabled=true;}
resetReviewEvidence();
let snapshot = null, selected = 0, review = null, polling = false, activeTab = 'activity', lastJob = null;
const drafts = new Map();let resumeAfter=null;
// DD-117: the coordinator row is selectable like a task row and shows its own work.
let coordinatorView=false;
let sessionStamp='', quotaAgent='',quotaActive='';const quotaRefreshed=new Map();
function statusLight(status){return ['running','completed','ready'].includes(status)?'green':['failed','blocked','permission_denied'].includes(status)?'red':'yellow';}
const pretty = value => {const key='state.'+String(value||'unknown');return Object.hasOwn(AC_MESSAGES.en,key)?acMessage(key):String(value||acMessage('room.unknown')).replaceAll('_',' ');};
// Run and task states: a capitalised label and a colour tone, so the final state reads at a glance.
const STATUS_TONE = {completed:'ok',approved:'ok',waiting_approval:'ok',merged:'ok',failed:'bad',blocked:'bad',permission_denied:'bad',rejected:'bad',running:'run',pending:'idle',ready:'idle'};
const statusTone = status => STATUS_TONE[status]||'wait';
// Literal labels so every state has an exact interface string to translate.
const STATUS_LABELS = {get completed(){return acMessage("startup.completed_22a970");},get failed(){return acMessage("room.failed_031a8f");},get pending(){return acMessage("room.pending");},get running(){return acMessage("room.running");},get ready(){return acMessage("startup.ready");},get blocked(){return acMessage("room.blocked");},get permission_denied(){return acMessage("room.permission_denied");},get stopped(){return acMessage("room.stopped");},get approved(){return acMessage("room.approved");},get rejected(){return acMessage("room.rejected");},get waiting_approval(){return acMessage("room.waiting_approval");},get waiting_tool_approval(){return acMessage("room.waiting_tool_approval");},get waiting_availability(){return acMessage("room.waiting_availability");},get interrupted(){return acMessage("room.interrupted");}};
const statusLabel = status => STATUS_LABELS[status]||(text=>text.charAt(0).toUpperCase()+text.slice(1))(pretty(status));
function node(tag, text, cls) { const el=document.createElement(tag);if(text!==undefined)AC_I18N.label(el,text);if(cls)el.className=cls;return el; }
function notice(message,error=false) { AC_I18N.text($('notice'),message);$('notice').className='notice'+(error?' error':'');$('notice').hidden=false;clearTimeout(notice.timer);notice.timer=setTimeout(()=>{$('notice').hidden=true;},error?11000:5000); }
// DD-40: installed application icons for built-in agents; custom agents keep letters.
let iconCache={},iconKinds='';const ICONS=()=>iconCache;
function avatar(parser,label,cls){const url=parser&&parser!=='custom'?ICONS()[parser]:null;if(url){const brand=window.acBrandAvatar?.(parser,cls);if(brand)return brand;const img=document.createElement('img');img.src=url;img.alt='';img.className=cls+' app-icon';img.setAttribute('aria-hidden','true');return img;}const span=document.createElement('span');span.className=cls;AC_I18N.bind(span,()=>((label||'?').slice(0,1).toUpperCase()));return span;}
async function request(packet, watched) {
  if (!watched && typeof AC_WAIT !== 'undefined') return AC_WAIT.watch(packet.kind === 'action' ? packet.action.kind : packet.kind, request(packet, true));
  const bridge=window.webkit?.messageHandlers?.agentconductor??(window.__acRequest?{postMessage:window.__acRequest}:null);
  if(!bridge)throw Error(acMessage("startup.native_bridge_unavailable"));
  // A timed-out mutation remains accepted. Never automatically retry it.
  let timer,visibility,remaining=15000,started=0;const began=Date.now();
  // Agent Room requests are marked so the workspace page can stay open beside the room.
  try {return JSON.parse(await Promise.race([bridge.postMessage(JSON.stringify({...packet,room:true})),new Promise((_,reject)=>{const resume=()=>{if(timer||document.hidden)return;started=performance.now();const deadline=started+remaining,grace=3750;timer=setTimeout(()=>{timer=null;if(document.hidden||performance.now()-deadline>grace){remaining=15000;resume();return;}try{Promise.resolve(bridge.postMessage(JSON.stringify({kind:'client_timeout',of:packet.kind,waited:Date.now()-began,hidden:document.hidden,room:true}))).catch(()=>{});}catch(_){}reject(Object.assign(Error(acMessage("startup.native_response_timed_out_refresh_before_retrying",{value1:packet.kind})),{transport:true}));},remaining);};const pause=()=>{if(!timer)return;clearTimeout(timer);timer=null;remaining=Math.max(1,remaining-(performance.now()-started));};visibility=()=>document.hidden?pause():resume();document.addEventListener('visibilitychange',visibility);resume();})]));}
  finally {clearTimeout(timer);if(visibility)document.removeEventListener('visibilitychange',visibility);}
}

function busy() { return snapshot?.job?.state==='running'; }
function idle() { return snapshot && !busy() && !['running','completed'].includes(snapshot.report.state); }
// A task that has not started can change agent or model while others run; the backend queues the change (#103).
function canEditTask(t) { return idle() || (snapshot?.report.state === 'running' && !busy() && t?.status === 'pending'); }
// A running task takes feedback that the backend queues for its next attempt (#95); otherwise the run must be idle.
// #165: the coordinator has its own draft and request; it needs a recorded coordinator and a run that is not executing.
const COORDINATOR_DRAFT = '__coordinator__';
function feedbackEnabled(task) { if (coordinatorView) return !busy() && !!snapshot.coordinator && !['running', 'stopped'].includes(snapshot.report.state); return !busy() && (snapshot.report.state === 'running' ? task?.status === 'running' : snapshot.report.state !== 'completed'); }
function task() { return snapshot?.report.tasks[selected]; }
// GitHub #122 (DD-172): guided recovery from an out-of-scope failure. Feedback never widens scope; an approved, previewed plan change does.
function scopeLines(text) { return String(text || '').split('\n').map(l => l.trim()).filter(Boolean); }
function scopeOffered(snap) { return !!snap && snap.report.state === 'failed' && /^out-of-scope change:/.test(snap.report.error || ''); }
let scopeFailure = null, scopeApproved = null;
function scopeListFill(id, items) { const ul = $(id); ul.replaceChildren(...items.map(text => { const li = document.createElement('li'); li.textContent = text; return li; })); }
function scopeInput() { const task = $('scope-task').value, current = scopeFailure?.tasks.find(t => t.id === task), test = scopeLines($('scope-test').value); return { task, paths: scopeLines($('scope-paths').value), test_command: current && JSON.stringify(test) !== JSON.stringify(current.test_command) && test.length ? test : undefined }; }
function scopeFillTask() { const t = scopeFailure.tasks.find(x => x.id === $('scope-task').value); $('scope-paths').value = [...new Set([...t.paths, ...scopeFailure.offending])].join('\n'); $('scope-test').value = t.test_command.join('\n'); scopeReset(); }
function scopeReset() { scopeApproved = null; $('scope-approve').disabled = true; $('scope-proposal').hidden = true; }
async function openScopeReview() {
  try {
    scopeFailure = await request({ kind: 'scope_failure' }); scopeReset();
    scopeListFill('scope-offending', scopeFailure.offending);
    scopeListFill('scope-tasks', scopeFailure.tasks.map(t => t.id + ': ' + t.paths.join(', ')));
    $('scope-stray-box').hidden = !scopeFailure.stray.length;
    scopeListFill('scope-stray', scopeFailure.stray.map(s => s.path + (s.exists_at_original ? ' — ' + acMessage('room.scope_stray_exists') : '')));
    $('scope-task').replaceChildren(...scopeFailure.tasks.map(t => { const o = document.createElement('option'); o.value = t.id; o.textContent = t.id; return o; }));
    $('scope-task').value = scopeFailure.tasks[0].id; scopeFillTask(); $('scope-dialog').showModal();
  } catch (e) { notice(e.message, true); }
}
async function previewScope() {
  try {
    const input = scopeInput(), preview = await request({ kind: 'scope_preview', task: input.task, paths: input.paths, ...(input.test_command ? { test_command: input.test_command } : {}) });
    const lines = [acMessage('room.scope_paths_change', { value1: preview.previous_paths.join(', '), value2: preview.paths.join(', ') })];
    if (JSON.stringify(preview.test_command) !== JSON.stringify(preview.previous_test_command)) lines.push(acMessage('room.scope_test_change', { value1: preview.previous_test_command.join(' '), value2: preview.test_command.join(' ') }));
    for (const r of preview.restore) lines.push(r.path + ' — ' + acMessage({ restore: 'room.scope_restore_restore', conflict: 'room.scope_restore_conflict', already_present: 'room.scope_restore_already_present' }[r.action] || 'room.scope_restore_conflict'));
    $('scope-proposal').textContent = lines.join('\n'); $('scope-proposal').hidden = false;
    scopeApproved = { ...input, digest: preview.digest }; $('scope-approve').disabled = false;
  } catch (e) { scopeReset(); notice(e.message, true); }
}
async function approveScope() {
  if (!scopeApproved) return;
  const { task, paths, test_command, digest } = scopeApproved;
  if (await act({ kind: 'scope_recovery', task, paths, ...(test_command ? { test_command } : {}), digest })) { $('scope-dialog').close(); notice(() => acMessage('room.scope_applied')); }
}
$('scope-review').onclick = openScopeReview; $('scope-preview').onclick = previewScope; $('scope-approve').onclick = approveScope;
$('scope-task').onchange = scopeFillTask; $('scope-paths').oninput = $('scope-test').oninput = scopeReset;
// The watched session finished (#109): follow the run to the next running session, or to the integration entry when the run awaits a decision.
// GitHub #688: entering a review state (also on the first snapshot of a run already waiting) opens the Integration verification entry, whichever task was shown.
// GitHub #735: a finished run opens on the coordinator's result card, like the review states.
function followTarget(prev, next, selectedId, onCoordinator) { if (onCoordinator) return null; if(isFailedVerification(next.report)&&prev?.report.state!=='failed')return {coordinator:true}; const review = ['waiting_approval', 'approved', 'rejected', 'completed']; if (review.includes(next.report.state) && (!prev || prev.report.state !== next.report.state)) return { coordinator: true }; if (!prev) return null; const was = prev.report.tasks.find(t => t.id === selectedId), now = next.report.tasks.find(t => t.id === selectedId); if (!was || was.status !== 'running' || !now || now.status === 'running') return null; const active = next.report.tasks.find(t => ['running', 'waiting_tool_approval'].includes(t.status)); if (active) return { id: active.id }; return review.includes(next.report.state) ? { coordinator: true } : null; }
function followProgress(prev) { const target = followTarget(prev, snapshot, task()?.id, coordinatorView); if (!target) return; if (target.coordinator) selectCoordinator(); else selectTask(snapshot.report.tasks.findIndex(t => t.id === target.id)); }
function formatAge(ms) { const s = Math.max(0, Math.floor(ms / 1000)); if (s < 60) return s + 's'; const m = Math.floor(s / 60); if (m < 60) return m + 'm ' + String(s % 60).padStart(2, '0') + 's'; return Math.floor(m / 60) + 'h ' + String(m % 60).padStart(2, '0') + 'm'; }
// Facts only (#100): the tool in progress, the age of the last output and the process state; silence is never called thinking or failure.
function activityLines(a, extraMs = 0) { if (!a) return []; const lines = []; if (a.tool) lines.push(acMessage("room.activity_tool", { value1: a.tool.name, value2: formatAge((a.tool.running_ms ?? 0) + extraMs) })); lines.push(a.quiet_ms == null ? acMessage("room.activity_no_output") : acMessage("room.activity_last_output", { value1: formatAge(a.quiet_ms + extraMs) })); const state = { present: 'room.activity_process_present', absent: 'room.activity_process_absent', unknown: 'room.activity_process_unknown' }[a.process]; if (state) lines.push(acMessage(state)); return lines; }
// GitHub #273 (DD-208): what a pending task waits for, the integration step in progress, and a stop or a lost run process.
function pendingReason(r, i, now) { const t = r.tasks[i], p = r.plan.tasks[i]; if (t?.status !== 'pending') return ''; const waits = (p.depends_on || []).filter(id => r.tasks.find(x => x.id === id)?.status !== 'completed'); if (waits.length) return acMessage("room.pending_dependencies", { value1: waits.join(', ') }); if (t.timeouts > 0 && t.retry_not_before * 1000 > now) return acMessage("room.pending_retry", { value1: formatAge(t.retry_not_before * 1000 - now) }); const a = r.plan.agents.find(x => x.id === p.agent); if (a && r.account_restrictions?.[a.quota_group || a.executable]) return acMessage("room.pending_account"); const active = r.tasks.filter((x, j) => x.status === 'running' && r.plan.tasks[j].agent === p.agent).length; if (a?.max_sessions && active >= a.max_sessions) return acMessage("room.pending_session_limit", { value1: a.max_sessions }); return ''; }
function verificationText(v, now) { if (!v) return ''; if (v.stage !== 'test') return acMessage("room.verification_preparing"); const out = v.output || {}; return acMessage("room.verification_test", { value1: v.index + 1, value2: v.total, value3: v.task, value4: formatAge(now - v.started_ms) }) + ' · ' + (out.updated_ms ? acMessage("room.activity_last_output", { value1: formatAge(now - out.updated_ms) }) : acMessage("room.activity_no_output")); }
function runAttention(s) { if (s?.report?.state !== 'running') return ''; if (s.stopping) return acMessage("room.stopping_run"); if (s.runner === 'absent' && s.job?.state !== 'running') return acMessage("room.runner_absent"); return ''; }
let activitySeen = null, activityAt = 0, lastSnapshotAt = 0, connectionWaited = false;
function renderActivity() { const el = $('task-activity'); if (!el) return; const a = snapshot?.activity?.[task()?.id]; if (snapshot !== activitySeen) { activitySeen = snapshot; activityAt = Date.now(); } const lines = coordinatorView ? [] : a ? activityLines(a, Date.now() - activityAt) : [pendingReason(snapshot.report, selected, Date.now())].filter(Boolean); const text = lines.join(' · '); el.style.visibility = text ? 'visible' : 'hidden'; if (el.textContent !== text) { el.textContent = text; el.title = text; } renderLive(); }
// Live facts are redrawn every second from recorded times, so a slow reply never freezes them; the age of the last snapshot is said after 5 s.
function renderLive() {
  if (!snapshot) return; const r = snapshot.report, now = Date.now(), extra = now - activityAt;
  for (const el of document.querySelectorAll('.session-note[data-task]')) { const i = r.tasks.findIndex(t => t.id === el.dataset.task), t = r.tasks[i], a = snapshot.activity?.[t?.id]; const text = t?.status === 'running' && a ? (a.quiet_ms == null ? acMessage("room.activity_no_output") : acMessage("room.activity_last_output", { value1: formatAge(a.quiet_ms + extra) })) : i >= 0 ? pendingReason(r, i, now) : ''; if (el.textContent !== text) el.textContent = text; el.hidden = !text; }
  AC_I18N.bind($('rail-elapsed'), () => railElapsed(r, now));
  const detail = $('final-detail'); if (detail && snapshot.verification) { const text = verificationText(snapshot.verification, now); if (detail.textContent !== text) detail.textContent = text; }
  const job = snapshot.job; if (AC_QUOTA_WAIT(job)) AC_I18N.bind($('quota-source'), () => AC_QUOTA_WAIT(snapshot.job)); AC_I18N.bind($('job-state'), () => (busy() && job.kind !== 'quota' ? pretty(job.kind) + acMessage("room.in_progress") + (job.started_ms ? ' · ' + formatAge(now - job.started_ms) : '') : job?.state === 'failed' && job.kind !== 'quota' ? acMessage("room.action_needs_attention") : ''));
  const stale = AC_STALE(lastSnapshotAt, now), waiting = AC_WAIT.text(now); if (stale || waiting) AC_I18N.bind($('connection'), () => stale || waiting); else if (connectionWaited && !pollFailures) AC_I18N.bind($('connection'), () => acMessage("room.connected_local_run")); connectionWaited = !!waiting && !stale; $('connection').classList.toggle('stale', !!stale);
  const box=$('run-attention'),attention=runAttention(snapshot)||stale;AC_I18N.text(box,attention);box.hidden=!attention;renderRunContext();
}
setInterval(() => { if (!document.hidden && !roomStopped) renderActivity(); }, 1000);
function planTask() { return snapshot?.report.plan.tasks[selected]; }
function profile() { return snapshot?.report.plan.agents.find(a=>a.id===planTask()?.agent); }
function agentName(agent) {return agent?acAgentName(agent,snapshot?.report?.plan?.agents||[]):acMessage("room.agent");}
// The plan review's one-line task summary: the first line that says what the task does.
function taskLine(prompt){const line=String(prompt||'').split('\n').map(l=>l.trim().replace(/^[-*]\s+/,'')).find(l=>l&&!/:$/.test(l)&&!/^<\/?[\w-]+>$/.test(l))||'';return line.length>160?line.slice(0,160)+'…':line;}
// Task rows lead with the reviewed summary; the task ID stays visible as a small label.
function taskHeading(id,prompt){const heading=node('span',undefined,'session-task'),summary=taskLine(prompt);heading.append(node('span',id,'task-id'));if(summary)heading.append(node('span',summary,'task-summary'));AC_I18N.bind(heading,()=>(summary?id+' · '+summary:id),"title");return heading;}
function refreshQuota(agent){quotaActive=agent;quotaRefreshed.set(agent,Date.now());return act({kind:'quota',agent});}
function renderQuota(agent) {
 setOptions($('quota-agent'),snapshot.report.plan.agents.map(a=>({value:a.id,label:agentName(a)})),quotaAgent||agent?.id,JSON.stringify(snapshot.report.plan.agents.map(a=>a.id)));
 agent=snapshot.report.plan.agents.find(a=>a.id===$('quota-agent').value)||agent;
 const quota=snapshot.quotas?.[agent?.id];
 AC_I18N.bind($('quota-refresh'),()=>(acMessage("room.refresh_quota")));
 AC_I18N.bind($('quota-refresh'),()=>(agent?.parser==='claude'?acMessage("room.on_macos_query_claude_usage_without_generation_otherwise"):acMessage("room.request_account_metadata_from_the_cli")),"title");
 // Budgets (Enterprise) read as a remaining percentage, like the agent cards; amounts stay below.
 const budgetRows=(snapshot.quota_budget_rows?.[agent?.id]||[]).map(b=>{const row=node('p',undefined,'quota-window');row.append(node('strong',b.headline));if(b.amounts)row.append(node('span',b.amounts,'small muted'));if(b.reset_label)row.append(node('span',b.reset_label,'small muted'));return row;});
 const windows=quota?.windows||[],labels=snapshot.quota_window_labels?.[agent?.id]||[];
 const resetLabels=snapshot.quota_window_reset_labels?.[agent?.id]||[];
 AC_I18N.replace($('quota-windows'),...budgetRows,...(windows.length?windows.map((w,index)=>{const row=node('p',undefined,'quota-window');const label=labels[index]||w.label;row.append(node('strong',()=>(`${typeof w.remaining_percent==='number'?acMessage("room.remaining", {value1:Math.round(w.remaining_percent)}):acMessage("room.unavailable")} · ${label}`)));if(resetLabels[index])row.append(node('span',resetLabels[index],'small muted'));return row;}):budgetRows.length?[]:[node('p',()=>(quota?.error||acMessage("room.account_quota_not_queried")),'small muted')]));
 AC_I18N.bind($('quota-source'),()=>(AC_QUOTA_WAIT(snapshot.job)||(quota?.observed_at?acMessage("room.observed_bcdb80", {value1:pretty(quota.status),value2:new Date(quota.observed_at*1000).toLocaleString(),value3:quota.source}):'')));
 if(typeof AC_PRESS!=='undefined')AC_PRESS.hold($('quota-refresh'),!!AC_QUOTA_WAIT(snapshot.job));$('quota-refresh').disabled=busy()||snapshot.report.state==='running'||!agent;
}
function selectTask(index) {
  drafts.set(coordinatorView?COORDINATOR_DRAFT:task()?.id,$('feedback').value);
  coordinatorView=false;selected=index;$('feedback').value=drafts.get(task()?.id)||'';renderSnapshot();loadTask();
}
function selectCoordinator() {
  drafts.set(coordinatorView?COORDINATOR_DRAFT:task()?.id,$('feedback').value);
  coordinatorView=true;$('feedback').value=drafts.get(COORDINATOR_DRAFT)||'';renderSnapshot();loadTask();
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
// inspector-fold (#146, amended by #394): the docked task detail column starts closed on every opening and is toggled only
// from the titlebar; a decision never opens it, it lights the toggle.
// A decision is open while the run needs a person; its key changes when the decision does.
function decisionKey(snap) {
  if (!snap || !snap.report) return '';
  const r = snap.report, pending = (snap.tool_requests || []).filter(q => q.status === 'pending').map(q => q.id);
  const waiting = ['waiting_approval', 'waiting_tool_approval', 'blocked'].includes(r.state) || (r.state === 'failed' && /^out-of-scope change:/.test(r.error || ''));
  return waiting || pending.length ? [r.state, r.candidate?.commit || '', pending.join(',')].join('|') : '';
}
// open = what is shown, key = the open decision.
function inspectorStep(s, ev) {
  if (ev.type === 'toggle') return { ...s, open: !s.open };
  return { ...s, key: ev.key || '' };
}
// inspector-fold-end
let inspectorState = { open: false, key: '' };
let inspectorHome = null;
function paintInspector() {
  const closed = !inspectorState.open;
  document.querySelector('.workbench').classList.toggle('insp-closed', closed);
  // GitHub #788: off the run screen the shared titlebar switch shows the Workspace panel's own state, not the Room's.
  const workspace = !document.body.classList.contains('on-run');
  for (const toggle of document.querySelectorAll('.inspector-switch')) {
    const shut = workspace && toggle.id === 'start-inspector-toggle' ? !document.querySelector('.startup-shell')?.classList.contains('insp-open') : closed;
    toggle.setAttribute('aria-expanded', String(!shut)); toggle.classList.toggle('on', !shut);
    AC_I18N.bind(toggle, () => acMessage(shut ? 'room.inspector_unfold' : 'room.inspector_fold'), 'aria-label');
    toggle.querySelector('.inspector-alert').hidden = !(shut && inspectorState.key);
  }
  // While a decision is open the candidate review (approve / reject) is the first section.
  const candidate = document.querySelector('.candidate-inspector'), heading = document.querySelector('.inspector-heading');
  if (!candidate) return;
  if (inspectorState.key && !inspectorHome) { inspectorHome = candidate.nextElementSibling; heading.after(candidate); }
  else if (!inspectorState.key && inspectorHome !== null) { inspectorHome.before(candidate); inspectorHome = null; }
}
function syncInspector() { inspectorState = inspectorStep(inspectorState, { type: 'snapshot', key: decisionKey(snapshot) }); paintInspector(); }
const toggleInspector = () => { inspectorState = inspectorStep(inspectorState, { type: 'toggle' }); paintInspector(); };
window.acInspector = { toggle: toggleInspector, paint: paintInspector }; $('inspector-toggle').onclick = toggleInspector;
paintInspector();
// DD-112: provider events read like the CLI transcript; raw JSON stays in the Raw logs tab.
const clipLines=(text,max=8)=>{const all=String(text??'').replace(/\s+$/,'').split('\n');return all.length>max?all.slice(0,max).join('\n')+acMessage("room.more_lines", {value1:all.length-max}):all.join('\n');};
const resultText=c=>typeof c==='string'?c:Array.isArray(c)?c.map(x=>x?.text||'').join('\n'):c?.text||'';
// Selects the task whose test failed on the combined result and drafts the direction to send.
function fixIntegration(test) {
  if(!test)return;const index=snapshot.report.tasks.findIndex(t=>t.id===test.task);if(index<0)return;
  selectTask(index);
  $('feedback').value=acMessage("room.the_coordinator_s_integration_verification_ran_this_task", {value1:test.output||acMessage("common.no_output")});
  $('feedback').focus();notice(()=>(acMessage("room.review_the_drafted_direction_then_send_feedback_the")));
}
// Every coordinator judgment is listed (#149); labels are explicit so each ID is checked.
const JUDGMENT_LABELS = {
  'tool:allow': () => acMessage("room.judgment_tool_allow"), 'tool:deny': () => acMessage("room.judgment_tool_deny"), 'tool:undecided': () => acMessage("room.judgment_tool_undecided"),
  'review:accept': () => acMessage("room.judgment_review_accept"), 'review:revise': () => acMessage("room.judgment_review_revise"), 'review:blocked': () => acMessage("room.judgment_review_blocked"),
  'test_command:replaced': () => acMessage("room.judgment_test_command_replaced"), 'timeout:resume': () => acMessage("room.judgment_timeout_resume"),
  'gate:passed': () => acMessage("room.judgment_gate_passed"), 'request:queued': () => acMessage("room.judgment_request_queued"), 'request:processing': () => acMessage("room.judgment_request_processing"), 'request:completed': () => acMessage("room.judgment_request_completed"), 'request:failed': () => acMessage("room.judgment_request_failed"), 'charter:recorded': () => acMessage("room.judgment_charter_recorded"), 'charter:updated': () => acMessage("room.judgment_charter_updated"), 'amendment:applied': () => acMessage("room.judgment_amendment_applied"), 'amendment:escalated': () => acMessage("room.judgment_amendment_escalated"), 'amendment:undone': () => acMessage("room.judgment_amendment_undone"), 'handoff:reassigned': () => acMessage("room.judgment_handoff_reassigned"), 'lesson:recorded': () => acMessage("room.judgment_lesson_recorded"),
};
function judgmentLabel(j) {
  if (j.kind === 'recovery') return acMessage("room.judgment_recovery", { value1: j.judgment });
  const label = JUDGMENT_LABELS[j.kind + ':' + j.judgment];
  return label ? label() : acMessage("room.judgment_other", { value1: j.kind, value2: j.judgment });
}
function judgmentEntry(j) {
  const el = event(judgmentLabel(j), (j.task ? j.task + ' · ' : '') + j.detail, ['deny', 'blocked', 'undecided', 'revise', 'escalated'].includes(j.judgment) ? 'warning' : 'status', j.seq);
  // An applied amendment can be undone once; the run stops for an explicit Resume.
  if (j.kind === 'amendment' && j.judgment === 'applied' && !j.undone) {
    const undo = node('button', () => acMessage("room.amendment_undo"), 'small-button');
    undo.onclick = async () => { try { await request({ kind: 'undo_amendment', id: j.id }); notice(() => acMessage("room.amendment_undone")); await poll(); } catch (e) { notice(e.message, true); } };
    el.querySelector('.event-body').append(undo);
  }
  return el;
}
function decisionEntry(d) {
  let input={};try{input=JSON.parse(d.input);}catch{}
  const label=d.allow===true?acMessage("room.coordinator_allowed"):d.allow===false?acMessage("room.coordinator_denied"):acMessage("room.coordinator_could_not_decide");
  const when=d.decided_ms?new Date(d.decided_ms).toLocaleTimeString()+' · ':'';
  return event(label,`${when}${toolLine(d.tool,input)}\n${d.reason||d.error||''}`,d.allow===true?'status':'warning');
}
function event(label,text,cls='',seq) {
  // GitHub #607: like the mock's `.ev` row: the number (or range) column first, then the title and its detail.
  const el=node('article',undefined,'event '+cls);el.append(node('code',seq===undefined?'':/^\d\d:\d\d$/.test(seq)?seq:'#'+seq,'event-seq'));const body=node('div',undefined,'event-body');const head=node('div',undefined,'event-head');head.append(node('span',label));body.append(head,node('p',text,'event-text'));el.append(body);return el;
}
async function loadTask() {if(!snapshot||roomStopped)return;if(coordinatorView){try{const data=await request({kind:'coordinator'});if(!roomStopped)renderCoordinator(data);}catch(e){notice(e.message,true);}return;}const index=selected;try{const data=await request({kind:'task',index});if(!roomStopped&&index===selected)renderTask(data);}catch(e){notice(e.message,true);}}
async function act(action,revision=snapshot?.revision) {
  if(roomStopped)return false;
  try{const accepted=await request({kind:'action',revision,action});if(action.kind!=='quota')notice(()=>(accepted.message||acMessage("room.accepted_waiting_for_the_coordinator", {value1:pretty(accepted.kind)})));await poll();return true;}catch(e){if(action.kind!=='quota')notice(e.message,true);await poll();return false;}
}
async function decision(title,description,evidence,needsReason=false,confirm=acMessage("room.confirm")) {
  const dialog=$('decision-dialog');AC_I18N.bind($('dialog-title'),()=>(title));AC_I18N.bind($('dialog-description'),()=>(description));AC_I18N.bind($('dialog-evidence'),()=>(evidence));
  $('dialog-reason').value='';$('dialog-reason').hidden=!needsReason;$('dialog-reason-label').hidden=!needsReason;$('dialog-reason').required=needsReason;AC_I18N.bind($('dialog-confirm'),()=>(confirm));dialog.returnValue='';dialog.showModal();
  return new Promise(resolve=>dialog.addEventListener('close',()=>resolve(dialog.returnValue==='confirm'?(needsReason?$('dialog-reason').value:true):null),{once:true}));
}
async function briefTeam(){return window.acBriefing.open({request,notice});}
async function loadReview() {
  try{const next=await request({kind:'evidence'});if(roomStopped)return;review=next;if(coordinatorView)reviewSummary(snapshot.report);AC_I18N.bind($('review-caption'),()=>(review.report.candidate?acMessage("room.candidate_exact_captured_evidence", {value1:review.report.candidate.commit.slice(0,12)}):acMessage("room.no_candidate_is_available")));
    AC_I18N.replace($('test-results'),...review.test_output.map(t=>{const el=node('article',undefined,'test-result'+(t.exit_code!==0||t.timed_out?' failed':''));el.append(node('strong',()=>(acMessage("room.exit", {value1:t.exit_code===0&&!t.timed_out?'✓':'×',value2:t.task,value3:t.exit_code??acMessage("room.unknown"),value4:t.timed_out?acMessage("common.timeout"):''}))));if(t.stdout||t.stderr)el.append(node('pre',t.stdout+'\n'+t.stderr));return el;}));
    AC_I18N.replace($('diff'),...review.diff.split('\n').map(line=>node('span',line||' ','diff-line'+(line.startsWith('+')?' add':line.startsWith('-')?' remove':line.startsWith('@@')?' hunk':''))));
    if(review.truncated){$('diff').append(node('span','…','diff-line muted'),node('span',()=>acMessage("room.full_diff_cli",{command:'agentconductor diff "'+snapshot.run_dir+'"'}),'diff-line muted'));notice(()=>acMessage("room.diff_exceeds_the_inline_review_limit_use_the"));}
    changeTab('review');renderSnapshot();
  }catch(e){notice(e.message,true);}
}
// GitHub #684: a live tool request is decided in the task card's footer (as in the mock), not in a modal; the card's notice shows the
// exact input. A request brings its task forward once; the direction field is replaced by the decision while one is pending.
let toolBusy=false;const seenTools=new Set();
function pendingTool(){const t=task();return t?(snapshot.tool_requests||[]).find(r=>r.status==='pending'&&r.context.task===t.id):null;}
function syncToolDialog() {
  const requests=snapshot.tool_requests||[],fresh=requests.find(r=>r.status==='pending'&&!seenTools.has(r.id));
  if(fresh){seenTools.add(fresh.id);const index=snapshot.report.tasks.findIndex(t=>t.id===fresh.context.task);if(index>=0&&index!==selected&&!coordinatorView)selectTask(index);}
  const waiting=coordinatorView?null:pendingTool(),box=$('tool-decision'),composer=document.querySelector('#run-screen .composer');
  const denied=renderDeniedFooter(waiting);box.hidden=!waiting;if(composer)composer.hidden=!!waiting||denied;
  if(!waiting)return;
  AC_I18N.bind($('tool-decision-note'),()=>acMessage("room.tool_decision_note"));
  // The paused worker belongs to the running job, so deciding is the live action that must stay available while that job runs.
  const live=snapshot.report.state==='running';
  for(const id of ['tool-decision-allow','tool-decision-deny'])$(id).disabled=!(idle()||live)||toolBusy;
}
async function decideTool(kind) {const waiting=pendingTool();if(!waiting||toolBusy)return;toolBusy=true;syncToolDialog();const idleRun=snapshot.report.state!=='running';try{if(await act({kind,sha:waiting.id})&&kind==='tool_approve'&&idleRun)resumeAfter='tool_approve';}finally{toolBusy=false;}syncToolDialog();}
wireReviewFooter();wireVerifyFooter();wireDeniedFooter();$('tool-decision-allow').onclick=()=>decideTool('tool_approve');$('tool-decision-deny').onclick=()=>decideTool('tool_reject');
function autoQuotaRefresh(){if(roomStopped||!snapshot||busy()||snapshot.report.state==='running')return;const agent=snapshot.report.plan.agents.find(a=>['claude','codex'].includes(a.parser)&&snapshot.quota_refresh_due?.[a.id]);if(agent)refreshQuota(agent.id);}
let pollFailures=0;function pollFailed(e,background){if(!e.transport||!background||++pollFailures>=3){AC_I18N.bind($('connection'),()=>(acMessage("room.local_connection_unavailable")));notice(e.message,true);}}function pollSucceeded(){pollFailures=0;lastSnapshotAt=Date.now();}
async function poll(background=false) {
  if(polling||roomStopped)return;polling=true;
  try{const kinds=JSON.stringify([...new Set((snapshot?.report?.plan?.agents||[]).map(a=>a.parser||'custom'))].sort());const next=await request({kind:'snapshot',quota_refresh_last:Object.fromEntries(quotaRefreshed),icons:!snapshot||kinds!==iconKinds});if(roomStopped)return;if(next.phase_transition&&window.acNextPhase?.follow){await window.acNextPhase.follow();return;}pollSucceeded();if(next.icons){iconCache=next.icons;iconKinds=JSON.stringify([...new Set((next.report?.plan?.agents||[]).map(a=>a.parser||'custom'))].sort());}const before=snapshot;snapshot=next;followProgress(before);const job=snapshot.job;
    if(snapshot.automatic_merge_error&&poll.autoMergeError!==snapshot.automatic_merge_error.id){poll.autoMergeError=snapshot.automatic_merge_error.id;notice(snapshot.automatic_merge_error.error||acMessage('native.automatic_merge_paused_gate'),true);}
    if(job&&job.state!=='running'&&lastJob!==job.id&&job.kind!=='quota'){lastJob=job.id;
      // Feedback or an approval sent to a run that is not running continues it.
      if(resumeAfter&&job.kind===resumeAfter){resumeAfter=null;if(job.state!=='failed'&&['ready','stopped'].includes(snapshot.report.state))setTimeout(()=>act({kind:'resume'}),0);}if(job.state==='failed')notice(()=>(job.error||(job.termination_signal?acMessage("room.action_terminated_by_signal_previous_evidence_is_retained", {value1:job.termination_signal}):acMessage("room.action_failed_previous_evidence_is_retained"))),true);else{
      notice(()=>(acMessage("room.finished_run", {value1:pretty(job.kind),value2:pretty(snapshot.report.state)})));
      if(job.kind==='approve'&&review&&review.report.candidate?.commit===snapshot.report.candidate?.commit&&review.report.diff_hash===snapshot.report.diff_hash&&JSON.stringify(review.report.tests)===JSON.stringify(snapshot.report.tests))review.revision=snapshot.revision;
      for(const t of snapshot.report.tasks)if(t.feedback&&drafts.get(t.id)===t.feedback){drafts.delete(t.id);if(task()?.id===t.id&&!composing&&$('feedback').value===t.feedback)$('feedback').value='';}
    }}renderSnapshot();setTimeout(autoQuotaRefresh,0);await loadTask();
  }catch(e){if(roomStopped)return;pollFailed(e,background);}finally{polling=false;}
}
for(const el of document.querySelectorAll('[data-tab]'))el.onclick=()=>el.dataset.tab==='review'?loadReview():changeTab(el.dataset.tab);
$('brief-team').onclick=briefTeam;$('quota-agent').onchange=()=>{quotaAgent=$('quota-agent').value;renderQuota(profile());};$('quota-refresh').onclick=()=>refreshQuota($('quota-agent').value);$('review-latest').onclick=loadReview;$('open-review').onclick=loadReview;$('goal-toggle').onclick=()=>{const open=$('goal-full').hidden;$('goal-full').hidden=!open;$('goal-toggle').setAttribute('aria-expanded',String(open));AC_I18N.bind($('goal-toggle').firstElementChild,()=>acMessage(open?'room.goal_collapse':'room.goal_expand'));};$('show-result').onclick=()=>openResult().catch(e=>notice(e.message,true));$('open-tools').onclick=()=>{const waiting=(snapshot?.tool_requests||[]).find(r=>r.status==='pending'),index=waiting?snapshot.report.tasks.findIndex(t=>t.id===waiting.context.task):-1;if(index>=0&&(index!==selected||coordinatorView))selectTask(index);(waiting?$('tool-decision'):$('tool-requests')).scrollIntoView({behavior:'smooth',block:'center'});};
// Input-method composition (e.g. Korean) must finish before text is replaced or sent.
let composing=false;document.addEventListener('compositionstart',()=>{composing=true;});document.addEventListener('compositionend',()=>{composing=false;});
$('feedback').oninput=()=>{const key=coordinatorView?COORDINATOR_DRAFT:task()?.id;if(key)drafts.set(key,$('feedback').value);};
$('send-feedback').onclick=async()=>{const feedback=$('feedback').value.trim();if(!feedback){notice(()=>(acMessage("room.enter_a_targeted_revision_before_sending")),true);return;}if(coordinatorView){if(await act({kind:'coordinator_request',message:feedback})){drafts.delete(COORDINATOR_DRAFT);$('feedback').value='';notice(()=>(acMessage("room.coordinator_request_sent")));}return;}const idleRun=snapshot.report.state!=='running';if(await act({kind:'revise',task:task().id,feedback})&&idleRun)resumeAfter='revise';};
$('agent-select').onchange=()=>renderSnapshot();$('reassign').onclick=()=>act({kind:'reassign',task:task().id,agent:$('agent-select').value,model:$('model-select').value||null});$('set-model').onclick=()=>act({kind:'model',task:task().id,model:$('model-select').value});
$('schedule').onclick=()=>{const date=new Date($('resume-at').value);if(!Number.isFinite(date.getTime())){notice(()=>(acMessage("room.choose_a_valid_local_date_and_time")),true);return;}act({kind:'schedule',task:task().id,at:date.toISOString().replace('.000Z','Z')});};
$('resume').onclick=()=>act({kind:'resume'});$('recover').onclick=async()=>{const revision=snapshot.revision;if(await decision(acMessage("room.recover_interrupted_run"),acMessage("room.recovery_refuses_any_live_or_unknown_recorded_process"),snapshot.run_dir))await act({kind:'recover'},revision);};
$('stop').onclick=async()=>{const reason=await decision(acMessage("room.stop_the_run"),acMessage("room.stop_all_active_tasks_and_tests_retain_their"),snapshot.report.plan.goal,true,acMessage("room.stop_run"));if(reason)await act({kind:'stop',reason});};
$('approve').onclick=async()=>{const packet=review;if(!packet||packet.revision!==snapshot.revision)return;if(await decision(acMessage("room.approve_verified_candidate"),acMessage("room.approve_exactly_the_diff_and_test_evidence_shown"),acMessage("room.candidate_diff_tests_verified", {value1:packet.report.candidate.commit,value2:packet.report.diff_hash,value3:packet.report.tests.length}),false,acMessage("room.approve")))await act({kind:'approve',sha:packet.report.candidate.commit},packet.revision);};
$('merge').onclick=async()=>{const packet=review;if(!packet)return;if(await decision(packet.report.direct_workspace?acMessage("room.accept_changes"):packet.report.folder?acMessage("room.apply_approved_changes"):acMessage("room.merge_approved_candidate"),packet.report.direct_workspace?acMessage("room.accept_existing_files"):packet.report.folder?acMessage("room.apply_the_exact_verified_changes_to_the_original"):acMessage("room.integrate_this_exact_approved_candidate_into_the_target"),acMessage("room.candidate", {value1:packet.report.folder?acMessage("room.folder")+packet.report.folder.original:acMessage("room.branch")+packet.report.branch,value2:packet.report.candidate.commit}),false,packet.report.direct_workspace?acMessage("room.accept_changes"):packet.report.folder?acMessage("room.apply_changes"):acMessage("room.merge")))await act({kind:'merge',sha:packet.report.candidate.commit},packet.revision);};
$('reject').onclick=async()=>{const revision=snapshot.revision;const reason=await decision(acMessage("room.reject_candidate"),snapshot.report.direct_workspace?acMessage("room.reject_restores_baseline"):acMessage("room.reject_the_current_integration_candidate_send_targeted_feedback"),snapshot.report.candidate?.commit||acMessage("room.no_candidate"),true,acMessage("room.reject"));if(reason)await act({kind:'reject',reason},revision);};
const roomShortcut=e=>{if((e.metaKey||e.ctrlKey)&&e.key==='Enter'&&!e.isComposing&&!composing&&!$('decision-dialog').open){e.preventDefault();if(!$('send-feedback').disabled)$('send-feedback').click();}};
document.addEventListener('keydown',roomShortcut);
// DD-71: 1 s while the run or an action is active, 5 s when idle, paused while hidden.
let roomTimer=null,roomStopped=false;function roomSchedule(){clearTimeout(roomTimer);if(roomStopped||document.hidden)return;const fast=snapshot?.job?.state==='running'||['running','ready','waiting_availability'].includes(snapshot?.report?.state);roomTimer=setTimeout(async()=>{await poll(true);roomSchedule();},fast?1000:5000);}
const roomVisibility=()=>{if(document.hidden)clearTimeout(roomTimer);else if(!roomStopped)poll(true).then(roomSchedule);};document.addEventListener('visibilitychange',roomVisibility);
const roomLanguage=()=>{if(!roomStopped&&snapshot){sessionStamp='';renderSnapshot();loadTask();if(review&&activeTab==='review')loadReview();}};document.addEventListener('ac-language-change',roomLanguage);
poll(true).then(roomSchedule);window.stopRoom=()=>{roomStopped=true;clearTimeout(roomTimer);document.removeEventListener('visibilitychange',roomVisibility);document.removeEventListener('keydown',roomShortcut);document.removeEventListener('ac-language-change',roomLanguage);};
