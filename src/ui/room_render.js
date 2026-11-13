// Shared run/task evidence rendering. Loaded in the same startRoom closure as room.js.
function renderSnapshot() {
  if(!snapshot)return;syncInspector();watchResult(snapshot);const r=snapshot.report;if(selected>=r.tasks.length)selected=0;
  const t=task(),p=planTask(),a=profile();

  AC_I18N.bind($('goal'),()=>(taskLine(r.plan.goal)||r.plan.goal));/* GitHub #827: the heading shows one line; the toggle unfolds the whole Goal. */const fullGoal=$('goal-full'),goalText=r.plan.goal||'';if(fullGoal.textContent!==goalText){fullGoal.hidden=true;$('goal-toggle').setAttribute('aria-expanded','false');AC_I18N.bind($('goal-toggle').firstElementChild,()=>acMessage('room.goal_expand'));}fullGoal.textContent=goalText;const project=(r.folder?.original||r.repo).replace(/^\\\\\?\\/,'');AC_I18N.bind($('repository'),()=>(project.split(/[\\/]/).filter(Boolean).pop()||project));AC_I18N.bind($('repository'),()=>(project),"title");
  AC_I18N.bind($('run-state'),()=>(statusLabel(r.state)));$('run-state').className='state-label tone-'+statusTone(r.state);
  renderRail(r,selected);
  const mergeRunning=snapshot.job?.state==='running'&&snapshot.job.kind==='merge';const asking=(snapshot.tool_requests||[]).some(q=>q.status==='pending'),stuck=r.tasks.some(x=>x.status==='permission_denied');AC_I18N.bind($('run-pill'),()=>(mergeRunning?acMessage("room.merge_in_progress"):asking?acMessage('room.decision_needed'):stuck?acMessage('room.blocked'):isVerifying(r)?acMessage('room.state_verifying'):statusLabel(r.state)));$('run-pill').className='status-pill tone-'+(mergeRunning?'run':asking?'wait':stuck?'bad':statusTone(r.state));
  const ph=snapshot.phase,phIndex=ph?(ph.phases||[]).findIndex(x=>x.id===ph.phase):-1;// DD-122: the titlebar names the run's phase and what it covers, beside the workspace.
  for(const id of ['start-run-phase','run-phase']){const box=document.getElementById(id);if(!box)continue;box.hidden=phIndex<0;AC_I18N.replace(box);if(phIndex>=0){box.append(node('strong',()=>(acMessage("room.phase", {value1:phIndex+1,value2:ph.phases.length,value3:ph.phases[phIndex].name||ph.phase}))),node('span',ph.description||'','run-phase-detail'));AC_I18N.bind(box,()=>(`${ph.slug||''} · ${ph.description||''}`),"title");}}
  const final=finalStep(r),stamp=JSON.stringify([selected,coordinatorView,final,r.tasks.map((t,i)=>[t.id,t.status,r.plan.tasks[i].agent]),r.plan.agents.map(a=>[a.id,a.parser]),snapshot.quotas,snapshot.job,Object.keys(snapshot.icons||{}),(snapshot.tool_requests||[]).filter(q=>q.status==='pending').map(q=>q.context.task)]);
  if(stamp!==sessionStamp){sessionStamp=stamp;window.acAgentStatus?.apply(r.tasks,r.plan.tasks,statusTone);AC_I18N.replace($('sessions'),...r.tasks.map((session,index)=>{
    const agent=r.plan.agents.find(a=>a.id===r.plan.tasks[index].agent),plan=r.plan.tasks[index]||{};const row=node('div',undefined,'session attn-'+statusTone(session.status)+(session.status==='running'?' is-running':'')+(selected===index&&!coordinatorView?' active':'')),button=node('button',undefined,'session-choice outline-task');button.setAttribute('aria-pressed',String(selected===index&&!coordinatorView));AC_I18N.bind(button,()=>(acMessage("room.select", {value1:agentName(agent),value2:session.id,value3:statusLabel(session.status)})),"aria-label");const symbol=avatar(agent?.parser,agentName(agent),'session-symbol');symbol.classList.add('outline-agent','st-'+statusLight(session.status));if(session.status==='running')symbol.classList.add('running');
    // GitHub #675: the row is the mock's outline task: agent mark, "T1 summary" with a status badge, a one-line scope, a chevron.
    const copy=node('span',undefined,'outline-copy'),head=node('span',undefined,'outline-task-heading'),title=node('strong'),asked=(snapshot.tool_requests||[]).some(q=>q.status==='pending'&&q.context.task===session.id),badge=node('span',asked?()=>acMessage('room.waiting_approval'):statusLabel(session.status),'outline-state-badge');title.append(node('code','T'+(index+1),'task-number'),' ',taskLine(plan.prompt)||'');badge.dataset.state=asked?'waiting':outlineState(session.status);if(asked||session.status==='permission_denied')symbol.classList.add('needs-attention',asked?'attn-wait':'attn-bad');head.append(title,badge);
    const scope=node('small',(plan.paths||[]).slice(0,2).join(', ')||'—','task-description'),note=node('small',undefined,'task-description session-note');note.dataset.task=session.id;note.hidden=true;
    AC_I18N.bind(button,()=>(session.id+' · '+(taskLine(plan.prompt)||session.id)),"title");copy.append(head,scope,note);button.append(symbol,copy,chevron());button.onclick=()=>selectTask(index);row.prepend(button);return row;
  }),finalRow(final));}
  const stage=r.state==='completed'?'complete':mergeRunning||r.state==='approved'||['waiting_approval','rejected'].includes(r.state)?'merge':isVerifying(r)||isFailedVerification(r)?'verify':r.tasks.some(t=>t.attempt>0)?'task':'plan';window.acStepper?.paint(stage,mergeRunning);window.acStepper?.available(true);
  // GitHub #580: the card head reads like the mock: the task summary leads, then agent · id · first owned path.
  AC_I18N.bind($('selected-agent'),()=>(taskLine(p?.prompt)||t?.id||agentName(a)));$('agent-symbol').replaceWith(Object.assign(avatar(a?.parser,agentName(a),'agent-symbol'),{id:'agent-symbol'}));AC_I18N.bind($('selected-task'),()=>([agentName(a),t?.id,p?.paths?.[0]].filter(Boolean).join(' · ')));AC_I18N.bind($('session-eyebrow'),()=>(acMessage("room.step_4_run_tasks_selected_task")));$('verify-rows').hidden=true;$('review-summary').hidden=true;renderNotice(t,a);AC_I18N.bind($('feedback-target'),()=>(agentName(a)+' / '+(t?.id||'')));AC_I18N.bind($('composer-note'),()=>acMessage("room.composer_note",{value1:agentName(a),value2:t?.id||''}));
  const toolWait=!!t&&(snapshot.tool_requests||[]).some(q=>q.status==='pending'&&q.context.task===t.id);AC_I18N.bind($('task-status'),()=>(toolWait?acMessage('room.tool_wait'):statusLabel(t?.status)));$('task-status').className='status-pill tone-'+(toolWait?'wait':statusTone(t?.status));
  AC_I18N.bind($('inspector-context'),()=>(t?agentName(a)+' · '+t.id+' · '+(p?.prompt||'').split('\n')[0]:acMessage("room.inspector_empty")));
  AC_I18N.bind($('task-instruction'),()=>(p?.prompt||''));AC_I18N.bind($('task-dependency'),()=>(p?.depends_on?.length?p.depends_on.join(', '):acMessage("room.no_dependencies")));AC_I18N.bind($('task-test'),()=>(p?.test_command?.join(' ')||'—'));
  AC_I18N.bind($('paths'),()=>(p?.paths.join('\n')||'—'));AC_I18N.bind($('session-id'),()=>(t?.session_id?acMessage("room.session_168693")+t.session_id:acMessage("room.no_native_session_yet")));
  setOptions($('agent-select'),r.plan.agents.map(a=>({value:a.id,label:agentName(a)})),p.agent,t.id+p.agent+JSON.stringify(r.plan.agents.map(a=>a.id)));
  // GitHub #852: the model list follows the agent picked in the select, so Assign can set both at once.
  const picked=r.plan.agents.find(x=>x.id===$('agent-select').value)||a,moved=!!picked&&picked.id!==p.agent,models=picked?.catalog?.models||[];setOptions($('model-select'),models.length?models.map(m=>({value:m,label:m})):[{value:'',label:acMessage("room.no_observed_catalog")}],moved?models[0]||'':p.model||models[0]||'',t.id+picked?.id+JSON.stringify(picked?.catalog)+String(p.model));
  const modelSelect=$('model-select'),manualTask=t.id;modelSelect.dataset.choiceIcon='model';modelSelect.dataset.choiceManual=String(!!a&&!moved&&canEditTask(t)&&r.state!=='running'&&(!models.length||a.catalog?.origin==='manual'));
  modelSelect.acManualPick=async value=>{if(!canEditTask(t))throw Error(acMessage('choices.agent_changed'));if(!await act({kind:'model',task:manualTask,model:value,register:true}))throw Error(acMessage('choices.registration_failed'));};
  AC_I18N.bind($('catalog-note'),()=>(a?.catalog?acMessage("room.observed", {value1:a.catalog.cli_version,value2:new Date(a.catalog.queried_at*1000).toLocaleDateString()}):acMessage("room.import_or_discover_a_model_catalog_with_the")));
  $('set-model').disabled=!canEditTask(t)||!models.length||moved;$('reassign').disabled=!canEditTask(t);$('schedule').disabled=!idle()||t.status!=='waiting_availability';
  const effectiveRestriction=t.restriction||snapshot.quotas?.[a?.id]?.restriction;
  AC_I18N.bind($('restriction'),()=>(effectiveRestriction?(effectiveRestriction.available_at?acMessage("room.available", {value1:new Date(effectiveRestriction.available_at*1000).toLocaleString()}):acMessage("room.reset_time_unknown_choose_a_time_below"))+' · '+((effectiveRestriction.source==='quota_observation'||effectiveRestriction.reason==='room.account_quota_exhausted')?acMessage("room.account_quota_exhausted"):String(effectiveRestriction.reason||effectiveRestriction.message||t.error||'').slice(0,180)):acMessage("room.no_recorded_restriction")));
  renderAvailability(t,a,snapshot.quotas?.[a?.id],(snapshot.availability_timer||[]).find(entry=>entry.task===t.id&&entry.agent===a?.id),snapshot.availability_timer_error);
  if(coordinatorView){const c=r.plan.agents.find(x=>x.id===snapshot.coordinator)||null;$('task-notice').hidden=true;AC_I18N.bind($('selected-agent'),()=>r.state==='completed'?acMessage("room.completed_review_title"):reviewMode(r)?acMessage("room.review_title"):acMessage("room.verify_title"));$('agent-symbol').replaceWith(Object.assign(node('span','◎','agent-symbol'),{id:'agent-symbol'}));AC_I18N.bind($('selected-task'),()=>r.state==='completed'?acMessage("room.completed_review_intro"):reviewMode(r)?acMessage("room.review_intro"):acMessage("room.verify_intro"));AC_I18N.bind($('session-eyebrow'),()=>(reviewMode(r)?acMessage("room.step_6_review_merge"):acMessage("room.step_5_run_verify")));AC_I18N.bind($('feedback-target'),()=>(acMessage("room.coordinator")));AC_I18N.bind($('composer-note'),()=>acMessage("room.composer_note_coordinator"));AC_I18N.bind($('task-status'),()=>(statusLabel(final.status)));$('task-status').className='status-pill tone-'+statusTone(final.status);AC_I18N.bind($('paths'),()=>('—'));AC_I18N.bind($('session-id'),()=>(c?acMessage("room.agent_8739d3", {value1:c.id}):''));}
  renderActivity();
  $('stop').disabled=!['running','ready','waiting_availability','waiting_tool_approval','stopped'].includes(r.state);
  $('resume').disabled=!idle()||['completed','approved','waiting_approval','rejected'].includes(r.state);$('resume').className=$('resume').disabled?'quiet':'primary';$('recover').disabled=busy()||r.state!=='running';$('recover').hidden=!(snapshot.interruptions?.length||['failed','blocked'].includes(r.state));
  // The button that started a UI job stays marked busy until that job ends.
  {const job=snapshot.job?.state==='running'?snapshot.job.kind:null;if(typeof AC_PRESS!=='undefined')for(const[id,kinds]of[['resume',['resume','auto_resume']],['approve',['approve']],['merge',['merge']],['recover',['recover']],['reject',['reject']]])AC_PRESS.hold($(id),kinds.includes(job));}
  $('send-feedback').disabled=!feedbackEnabled(t);AC_I18N.bind($('feedback'),()=>(coordinatorView?acMessage("room.send_a_request_to_the_coordinator"):acMessage("room.request_a_change_point_out_an_edge_case")),"placeholder");
  AC_I18N.bind($('completion'),()=>(`${r.tasks.filter(t=>t.status==='completed').length} / ${r.tasks.length}`));
  AC_I18N.bind($('candidate-sha'),()=>(r.candidate?.commit||acMessage("room.no_candidate")));AC_I18N.bind($('test-count'),()=>(r.tests.length?acMessage("room.passed", {value1:r.tests.filter(t=>t.exit_code===0&&!t.timed_out).length}):'—'));
  const currentReview=review&&review.revision===snapshot.revision;
  $('review-stale').hidden=!review||review.revision===snapshot.revision;
  $('approve').disabled=!idle()||r.state!=='waiting_approval'||!currentReview;$('reject').disabled=!idle()||!r.candidate;
  $('merge').disabled=!idle()||r.state!=='approved'||!currentReview;
  AC_I18N.bind($('merge'),()=>(r.direct_workspace?acMessage("room.accept_changes"):r.folder?acMessage("room.apply_changes"):acMessage("room.merge")));
  AC_I18N.bind($('candidate-note'),()=>(r.direct_workspace?(r.state==='completed'?acMessage("room.accepted_complete"):r.state==='approved'?acMessage("room.approved_accept_changes"):acMessage("room.review_diff_and_tests_before_approval")):r.folder?(r.state==='completed'?acMessage("room.changes_applied_this_run_is_complete"):r.state==='approved'?acMessage("room.approved_applying_changes_remains_a_separate_action"):acMessage("room.review_diff_and_tests_before_approval")):r.state==='completed'?acMessage("room.merged_this_run_is_complete"):r.state==='approved'?acMessage("room.approved_merging_remains_a_separate_action"):review&&!currentReview?acMessage("room.state_changed_inspect_the_latest_evidence"):acMessage("room.review_diff_and_tests_before_approval")));
  renderReviewFooter(r);renderVerifyFooter(r);renderGoalInfo(r);renderRunRecovery(r);renderQuota(a);renderTools();{const g=document.querySelector('.gates-nav');$('open-review').hidden=!['waiting_approval','approved','completed','rejected'].includes(r.state);$('open-tools').hidden=Number($('tool-count').textContent)===0;g.hidden=[...g.querySelectorAll('button')].every(b=>b.hidden);}acStatusRepository.render($('connection').parentElement,window.acRepositoryIdentity);AC_I18N.bind($('connection'),()=>((typeof AC_WAIT!=='undefined'&&AC_WAIT.text())||acMessage("room.connected_local_run")));
}
function toolLine(name,input){const i=input&&typeof input==='object'?input:{};const arg=i.command??i.file_path??i.filePath??i.path??i.pattern??i.url??i.query??'';return `● ${name}(${String(arg).split('\n')[0].slice(0,200)})`+(i.description&&i.command?`\n  ${i.description}`:'');}
function transcript(e) {
  const raw=e.payload&&typeof e.payload==='object'?e.payload:{},out=[],add=(label,text,cls='')=>out.push({label,text:String(text??''),cls});
  const it=raw.item||{},done=raw.type==='item.completed';
  switch(raw.type){
    case 'system':add(acMessage("room.session"),raw.subtype==='init'?acMessage("startup.started")+(raw.model?' · '+raw.model:''):statusLabel(raw.subtype),'status');break;
    case 'assistant':for(const c of raw.message?.content||[]){if(c.type==='text'&&c.text?.trim())add(acMessage("room.agent"),c.text);else if(c.type==='tool_use')add(acMessage("room.tool"),toolLine(c.name,c.input),'tool');else if(c.type==='thinking'&&c.thinking)add(acMessage("room.thinking"),clipLines(c.thinking,4),'muted');}break;
    case 'user':for(const c of raw.message?.content||[])if(c.type==='tool_result')add(c.is_error?acMessage("room.tool_error"):acMessage("room.result"),clipLines(resultText(c.content))||acMessage("common.no_output"),c.is_error?'error':'result');break;
    case 'result':add(raw.is_error?acMessage("room.failed_031a8f"):acMessage("room.finished"),typeof raw.result==='string'?raw.result:raw.subtype?statusLabel(raw.subtype):'',raw.is_error?'error':'status');break;
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
  const row=node('div',undefined,'session final-step attn-'+statusTone(final.status)+(coordinatorView?' active':'')),button=node('button',undefined,'session-choice outline-task');button.setAttribute('aria-pressed',String(coordinatorView));
  AC_I18N.bind(button,()=>(acMessage("room.coordinator_integration_verification", {value1:statusLabel(final.status)})),"aria-label");
  const symbol=node('span','◎','session-symbol outline-agent');symbol.setAttribute('aria-hidden','true');
  const copy=node('span',undefined,'outline-copy'),head=node('span',undefined,'outline-task-heading'),badge=node('span',statusLabel(final.status),'outline-state-badge');badge.dataset.state=outlineState(final.status);
  head.append(node('strong',()=>(acMessage("room.integration_verification"))),badge);
  copy.append(head,Object.assign(node('small',final.detail,'task-description'),{id:'final-detail'}),node('small',()=>(acMessage("room.decisions_count", {value1:snapshot.decision_count||0})),'task-description'));
  button.append(symbol,copy,chevron());button.onclick=selectCoordinator;row.append(button);return row;
}
// GitHub #267 (DD-206): the coordinator tab starts with the run's status, and the `workflow` rows (one per report save, always the same counts) are not drawn, because the board carries that status; the stored history is unchanged.
function boardCounts(tasks) { const n = s => tasks.filter(t => t.status === s).length; return { total: tasks.length, completed: n('completed'), running: n('running'), failed: n('failed'), pending: n('pending') }; }
function shownHistory(rows) { return rows.filter(h => h.event !== 'workflow'); }
function boardNode(r) { const c = boardCounts(r.tasks), box = node('section', undefined, 'coord-board'); box.append(node('p', () => acMessage("room.board_summary", { value1: statusLabel(r.state), value2: c.completed, value3: c.total, value4: c.running, value5: c.failed, value6: c.pending }), 'coord-board-summary')); r.tasks.forEach((t, i) => { const act = activityLines(snapshot.activity?.[t.id], Date.now() - activityAt).join(' · ') || pendingReason(r, i, Date.now()); box.append(node('p', () => [t.id, r.plan.tasks[i]?.agent, statusLabel(t.status), act].filter(Boolean).join(' · '), 'coord-board-task')); }); return box; }
function renderCoordinator(data) {
  if(!coordinatorView)return;const r=snapshot.report;verifyRows(r);reviewSummary(r);if(reviewMode(r)&&r.candidate&&!review)loadReview(false);
  const timeline=[event(acMessage("room.coordinator"),acMessage("room.runs_every_task_test_on_the_combined_result"),'plan')];
  const judged=new Map((data.judgments||[]).map(j=>[j.seq,j]));timeline.push(boardNode(r));for(const h of shownHistory(data.history||[])){if(h.event==='tool_orchestrator_decision'){timeline.push(decisionEntry(h.payload));continue;}if(judged.has(h.seq)){timeline.push(judgmentEntry(judged.get(h.seq)));continue;}const bad=/fail|error|reject|interrupt/i.test(h.event+' '+h.summary);timeline.push(event(statusLabel(h.event),h.summary||'',bad?'error':'status',h.seq));}
  for(const t of r.tests){const ok=t.exit_code===0&&!t.timed_out;timeline.push(event(ok?acMessage("room.integration_test_passed"):acMessage("room.integration_test_failed"),acMessage("room.exit_df653f", {value1:t.task,value2:t.exit_code??'?',value3:t.timed_out?acMessage("common.timed_out"):'',value4:t.output?'\n'+clipLines(t.output,12):''}),ok?'result':'error'));}
  if(r.error)timeline.push(event(acMessage("room.run_error"),r.error,'error'));
  const activity=$('activity'),nearBottom=activity.parentElement.scrollHeight-activity.parentElement.scrollTop-activity.parentElement.clientHeight<70;
  AC_I18N.replace(activity,...timeline);if(nearBottom)activity.parentElement.scrollTop=activity.parentElement.scrollHeight;
  AC_I18N.bind($('stdout'),()=>(readableRawLog(data.stdout)||acMessage("room.no_verification_output_yet")));AC_I18N.bind($('stderr'),()=>(readableRawLog(data.stderr)||acMessage("room.no_errors_logged")));
  AC_I18N.replace($('lessons'),node('p',()=>(acMessage("room.lessons_belong_to_tasks_select_a_task_to")),'empty'));
}
function renderTask(data) {
  const t=task(),p=planTask();if(!t||data.task.id!==t.id)return;
  const started=Number.isFinite(t.started_ms)&&t.started_ms>0?new Date(t.started_ms).toLocaleTimeString([],{hour:"2-digit",minute:"2-digit",hour12:false}):undefined;const timeline=[event(acMessage("room.assigned_task"),p.prompt,'plan',started)];if(t.feedback)timeline.push(event(acMessage("room.you_latest_review"),t.feedback));
  for(const x of collapseThinking(data.events)){if(x.thinking){const t=x.thinking;timeline.push(event(acMessage("room.thinking_updates",{value1:t.count}),acMessage("room.thinking_quiet"),'muted',t.first===t.last?t.first:`${t.first}–${t.last}`));}else for(const y of transcript(x.event))timeline.push(event(y.label,y.text,y.cls,x.event.at?new Date(x.event.at).toLocaleTimeString([],{hour:'2-digit',minute:'2-digit',hour12:false}):x.event.seq));}
  for(const d of data.decisions||[])timeline.push(decisionEntry(d));
  for(const d of data.coordinator_decisions||[])if(d.kind!=='tool')timeline.push(judgmentEntry(d));
  if(t.error)timeline.push(event(acMessage("room.coordinator_report"),t.error,'error'));
  if(!data.events.length)timeline.push(node('p',()=>(acMessage("room.no_provider_events_have_been_reported_yet")),'empty'));
  const activity=$('activity'),nearBottom=activity.parentElement.scrollHeight-activity.parentElement.scrollTop-activity.parentElement.clientHeight<70;
  AC_I18N.replace(activity,...timeline);if(nearBottom)activity.parentElement.scrollTop=activity.parentElement.scrollHeight;
  AC_I18N.bind($('stdout'),()=>(readableRawLog(data.stdout)||acMessage("room.no_output_yet")));AC_I18N.bind($('stderr'),()=>(readableRawLog(data.stderr)||acMessage("room.no_errors_logged")));
  const notes=data.knowledge.filter(k=>k.task===t.id);const steps=(data.step_lessons||[]).map(l=>{const el=node('article',undefined,'lesson');el.append(node('h3',()=>(acMessage("room.step_lesson_for",{value1:l.task}))),node('p',l.cause,'small muted'),node('p',l.response));return el;});AC_I18N.replace($('lessons'),...steps,...(notes.length?notes.map(k=>{const el=node('article',undefined,'lesson');el.append(node('h3',k.lesson.context),node('span',()=>(acMessage("room.confidence", {value1:pretty(k.status),value2:k.lesson.confidence})),'status-pill'),node('p',k.lesson.response),node('pre',k.lesson.sources.map(s=>s.path+'\n'+s.sha256).join('\n')),node('p',()=>(acMessage("room.revalidate")+k.lesson.revalidate),'small muted'));return el;}):(steps.length?[]:[node('p',()=>(acMessage("room.no_reviewed_lessons_for_this_task_draft_observations")),'empty')])));
  const u=data.usage;AC_I18N.bind($('tokens'),()=>(u&&typeof u.input_tokens==='number'?acMessage("room.in_out", {value1:u.input_tokens.toLocaleString(),value2:(u.output_tokens||0).toLocaleString()}):acMessage("room.not_reported")));
  const duration=t.started_ms?(Number(t.finished_ms||Date.now())-Number(t.started_ms))/1000:null;AC_I18N.bind($('duration'),()=>(duration===null?'—':`${Math.max(0,duration).toFixed(1)}s`));
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
