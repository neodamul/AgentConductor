'use strict';
const $ = id => document.getElementById(id);
let snapshot = null, selected = 0, review = null, polling = false, activeTab = 'activity', lastJob = null;
const drafts = new Map();
let sessionStamp='', quotaAgent='',quotaActive='';const quotaRefreshed=new Map();
function statusLight(status){return ['running','completed','ready'].includes(status)?'green':['failed','blocked'].includes(status)?'red':'yellow';}
const names = {claude:'Claude Code',codex:'Codex',opencode:'OpenCode'};
const pretty = value => String(value || 'unknown').replaceAll('_',' ');
function node(tag, text, cls) { const el=document.createElement(tag);if(text!==undefined)el.textContent=text;if(cls)el.className=cls;return el; }
function notice(message,error=false) { $('notice').textContent=message;$('notice').className='notice'+(error?' error':'');$('notice').hidden=false;clearTimeout(notice.timer);notice.timer=setTimeout(()=>{$('notice').hidden=true;},error?11000:5000); }
// DD-40: installed application icons for built-in agents; custom agents keep letters.
let iconCache={},iconKinds='';const ICONS=()=>iconCache;
function avatar(parser,label,cls){const url=parser&&parser!=='custom'?ICONS()[parser]:null;if(url){const img=document.createElement('img');img.src=url;img.alt='';img.className=cls+' app-icon';img.setAttribute('aria-hidden','true');return img;}const span=document.createElement('span');span.className=cls;span.textContent=(label||'?').slice(0,1).toUpperCase();return span;}
async function request(packet) {
  const bridge=window.webkit?.messageHandlers?.agentconductor??(window.__acRequest?{postMessage:window.__acRequest}:null);
  if(!bridge)throw Error('Native bridge unavailable');
  // A timed-out mutation remains accepted. Never automatically retry it.
  let timer;
  try {return JSON.parse(await Promise.race([bridge.postMessage(JSON.stringify(packet)),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('Native response timed out; refresh before retrying')),15000);})]));}
  finally {clearTimeout(timer);}
}

function busy() { return snapshot?.job?.state==='running'; }
function idle() { return snapshot && !busy() && !['running','completed'].includes(snapshot.report.state); }
function task() { return snapshot?.report.tasks[selected]; }
function planTask() { return snapshot?.report.plan.tasks[selected]; }
function profile() { return snapshot?.report.plan.agents.find(a=>a.id===planTask()?.agent); }
function agentName(agent) {return names[agent?.parser]||agent?.id||'Agent';}
function quotaBadge(quota) {const windows=[...(quota?.windows||[])].sort((a,b)=>(a.window_minutes||({five_hour:300,seven_day:10080}[a.label])||1e9)-(b.window_minutes||({five_hour:300,seven_day:10080}[b.label])||1e9)),enterprise=quota?.plan_type?.toLowerCase()==='enterprise',label=w=>enterprise&&['primary','five_hour'].includes(w.label)?'Current':enterprise&&['secondary','seven_day'].includes(w.label)?'Total':w.label==='five_hour'?'5h':w.label==='seven_day'?'Weekly':w.label==='seven_day_opus'?'Weekly Opus':w.label==='seven_day_sonnet'?'Weekly Sonnet':w.window_minutes===1440?'Daily':w.window_minutes===10080?'Weekly':w.window_minutes&&w.window_minutes%1440===0?`${w.window_minutes/1440}d`:w.window_minutes&&w.window_minutes%60===0?`${w.window_minutes/60}h`:w.window_minutes?`${w.window_minutes}m`:w.label||'window',valid=w=>typeof w.remaining_percent==='number'&&Number.isFinite(w.remaining_percent),stale=quota?.status==='stale',text=stale?'Stale':windows.some(valid)?windows.map(w=>valid(w)?Math.round(w.remaining_percent)+'%':'—').join('/'):'—';const title=(stale?'Quota observation is stale. ':'Remaining account capacity. ')+windows.map(w=>`${label(w)}: ${valid(w)?Math.round(w.remaining_percent)+'% remaining':'unavailable'}`).join(' / ')+(quota?.plan_type?' · '+quota.plan_type:'')+(quota?.error?' · '+quota.error:'')+(quota?.source?' · '+quota.source:'')+(quota?.observed_at?' · observed '+new Date(quota.observed_at*1000).toLocaleString():'');return {text,title};}
function quotaRefreshDue(last,now) {return last===undefined||now<last||now-last>=120000;}
function refreshQuota(agent){quotaActive=agent;quotaRefreshed.set(agent,Date.now());return act({kind:'quota',agent});}
function quotaWindowLabel(w) {return w.label==='seven_day_opus'?'Weekly Opus':w.label==='seven_day_sonnet'?'Weekly Sonnet':w.window_minutes===10080?'Weekly':w.window_minutes===300?'5-hour':w.window_minutes?`${w.window_minutes}-minute`:pretty(w.label);}
function quotaSummary(quota) {const windows=(quota?.windows||[]).filter(w=>typeof w.remaining_percent==='number');return windows.length?windows.map(w=>`${Math.round(w.remaining_percent)}% left · ${quotaWindowLabel(w)}`).join(' / '):quota?.status==='stale'?'Quota stale':'Quota unavailable';}
function renderQuota(agent) {
 setOptions($('quota-agent'),snapshot.report.plan.agents.map(a=>({value:a.id,label:agentName(a)+' · '+a.id})),quotaAgent||agent?.id,JSON.stringify(snapshot.report.plan.agents.map(a=>a.id)));
 agent=snapshot.report.plan.agents.find(a=>a.id===$('quota-agent').value)||agent;
 const quota=snapshot.quotas?.[agent?.id];
 $('quota-refresh').textContent='Refresh quota';
 $('quota-refresh').title=agent?.parser==='claude'?'On macOS, query Claude /usage without generation; otherwise check owned stream metadata.':'Request account metadata from the CLI.';
 const windows=quota?.windows||[];
 $('quota-windows').replaceChildren(...(windows.length?windows.map(w=>{const row=node('p',undefined,'quota-window');const label=quotaWindowLabel(w);row.append(node('strong',`${typeof w.remaining_percent==='number'?`${Math.round(w.remaining_percent)}% remaining`:'Unavailable'} · ${label}`));if(w.resets_at)row.append(node('span','Resets '+new Date(w.resets_at*1000).toLocaleString(),'small muted'));return row;}):[node('p',quota?.error||'Account quota not queried.','small muted')]));
 $('quota-source').textContent=quota?.observed_at?`${pretty(quota.status)} · observed ${new Date(quota.observed_at*1000).toLocaleString()} · ${quota.source}`:'';
 $('quota-refresh').disabled=busy()||snapshot.report.state==='running'||!agent;
}
function selectTask(index) {
  if(task())drafts.set(task().id,$('feedback').value);
  selected=index;$('feedback').value=drafts.get(task()?.id)||'';renderSnapshot();loadTask();
}
function changeTab(tab) {
  activeTab=tab;for(const el of document.querySelectorAll('[data-tab]')){const active=el.dataset.tab===tab;el.classList.toggle('active',active);el.setAttribute('aria-selected',String(active));}
  for(const name of ['activity','review','logs','knowledge'])$(name+'-view').hidden=name!==tab;
}
function setOptions(select,values,current,stamp) {
  if(select.dataset.stamp===stamp)return;const chosen=document.activeElement===select?select.value:current;
  select.replaceChildren(...values.map(value=>{const option=node('option',value.label);option.value=value.value;return option;}));
  if(values.some(v=>v.value===chosen))select.value=chosen;select.dataset.stamp=stamp;
}
function renderSnapshot() {
  if(!snapshot)return;const r=snapshot.report;if(selected>=r.tasks.length)selected=0;
  const t=task(),p=planTask(),a=profile();
  $('goal').textContent=r.plan.goal;$('repository').textContent=r.repo.split('/').filter(Boolean).pop()||r.repo;
  $('workspace-mode').textContent=r.plan.workspace_mode==='worktree'?'Isolated worktrees':'Shared directory';$('branch').textContent=r.branch;
  $('run-state').textContent=pretty(r.state);$('session-count').textContent=r.tasks.length;
  const stamp=JSON.stringify([selected,r.tasks.map((t,i)=>[t.id,t.status,r.plan.tasks[i].agent]),r.plan.agents.map(a=>[a.id,a.parser]),snapshot.quotas,snapshot.job,Object.keys(snapshot.icons||{})]);
  if(stamp!==sessionStamp){sessionStamp=stamp;$('sessions').replaceChildren(...r.tasks.map((session,index)=>{
    const agent=r.plan.agents.find(a=>a.id===r.plan.tasks[index].agent);const row=node('div',undefined,'session'+(selected===index?' active':'')),button=node('button',undefined,'session-choice');button.setAttribute('aria-label',`Select ${agentName(agent)} ${session.id}: ${pretty(session.status)}`);const light=node('span',undefined,'agent-light '+statusLight(session.status)+(session.status==='running'?' running':''));light.setAttribute('aria-hidden','true');button.append(light,avatar(agent?.parser,agentName(agent),'session-symbol'));
    const body=node('span',undefined,'session-body'),heading=node('span',undefined,'name-row');heading.append(node('span',agentName(agent),'name'));const controls=node('span',undefined,'quota-controls');if(['claude','codex'].includes(agent?.parser)){const value=quotaBadge(snapshot.quotas?.[agent.id]),badge=node('span',value.text,'quota-badge');badge.title=value.title;badge.setAttribute('aria-label',value.title);controls.append(badge);}const refresh=node('button','↻','quota-refresh');refresh.type='button';refresh.title=`Refresh ${agentName(agent)} quota`;refresh.setAttribute('aria-label',refresh.title);refresh.disabled=busy()||r.state==='running'||!['claude','codex'].includes(agent?.parser);if(busy()&&quotaActive===agent?.id)refresh.classList.add('refreshing');refresh.onclick=e=>{e.stopPropagation();refreshQuota(agent.id);};controls.append(refresh);row.append(controls);body.append(heading,node('span',session.id,'sub'),node('span',pretty(session.status),'sub'));button.append(body);button.onclick=()=>selectTask(index);row.prepend(button);return row;
  }));}
  const stage=r.state==='completed'?'complete':['waiting_approval','approved','rejected'].includes(r.state)?'verify':r.tasks.some(t=>t.attempt>0)?'task':'plan';const stages=['plan','task','verify','complete'];for(const el of document.querySelectorAll('[data-stage]')){el.classList.toggle('current',el.dataset.stage===stage);el.classList.toggle('done',stages.indexOf(el.dataset.stage)<stages.indexOf(stage));}
  $('selected-agent').textContent=agentName(a);$('agent-symbol').replaceWith(Object.assign(avatar(a?.parser,agentName(a),'agent-symbol'),{id:'agent-symbol'}));$('selected-task').textContent=t?.id||'';$('feedback-target').textContent=agentName(a)+' / '+(t?.id||'');
  $('task-status').textContent=pretty(t?.status);$('task-status').className='status-pill'+(t?.status.includes('waiting')?' waiting':'')+(t?.status==='failed'?' failed':'');
  $('paths').textContent=p?.paths.join('\n')||'—';$('session-id').textContent=t?.session_id?'Session '+t.session_id:'No native session yet';
  setOptions($('agent-select'),r.plan.agents.map(a=>({value:a.id,label:agentName(a)+' · '+a.id})),p.agent,t.id+p.agent+JSON.stringify(r.plan.agents.map(a=>a.id)));
  const models=a?.catalog?.models||[];setOptions($('model-select'),models.length?models.map(m=>({value:m,label:m})):[{value:'',label:'No observed catalog'}],p.model||models[0]||'',t.id+JSON.stringify(a?.catalog)+String(p.model));
  $('catalog-note').textContent=a?.catalog?`${a.catalog.cli_version} · observed ${new Date(a.catalog.queried_at*1000).toLocaleDateString()}`:'Import or discover a model catalog with the CLI.';
  $('set-model').disabled=!idle()||!models.length;$('reassign').disabled=!idle();$('schedule').disabled=!idle()||t.status!=='waiting_availability';
  $('restriction').textContent=t.restriction?(t.restriction.available_at?`Available ${new Date(t.restriction.available_at*1000).toLocaleString()}`:'Reset time unknown; choose a time below.')+' · '+String(t.restriction.reason||t.restriction.message||t.error||'').slice(0,180):'No recorded restriction.';
  $('stop').disabled=!['running','ready','waiting_availability','waiting_tool_approval','stopped'].includes(r.state);
  $('resume').disabled=!idle()||['completed','approved','waiting_approval','rejected'].includes(r.state);$('recover').disabled=busy()||r.state!=='running';
  $('send-feedback').disabled=!idle()||['completed'].includes(r.state);
  $('job-state').textContent=busy()?pretty(snapshot.job.kind)+' in progress…':snapshot.job?.state==='failed'?'Action needs attention':'';
  $('completion').textContent=`${r.tasks.filter(t=>t.status==='completed').length} / ${r.tasks.length}`;
  $('candidate-sha').textContent=r.candidate?.commit||'No candidate';$('test-count').textContent=r.tests.length?`${r.tests.filter(t=>t.exit_code===0&&!t.timed_out).length} passed`:'—';
  const currentReview=review&&review.revision===snapshot.revision&&!review.truncated;
  $('review-stale').hidden=!review||review.revision===snapshot.revision;
  $('approve').disabled=!idle()||r.state!=='waiting_approval'||!currentReview;$('reject').disabled=!idle()||!r.candidate;
  $('merge').disabled=!idle()||r.state!=='approved'||!currentReview;
  $('candidate-note').textContent=r.state==='completed'?'Merged; this run is complete.':r.state==='approved'?'Approved; merging remains a separate action.':review&&!currentReview?'State changed; inspect the latest evidence.':'Review diff and tests before approval.';
  renderQuota(a);renderTools();$('run-path').textContent=snapshot.run_dir;$('connection').textContent='Connected · local run';
}
function eventText(event) {
  let raw;try{raw=JSON.parse(event.text);}catch{return event.text;}
  if(raw.item?.text)return raw.item.text;if(raw.item?.message)return raw.item.message;
  if(raw.result)return typeof raw.result==='string'?raw.result:JSON.stringify(raw.result);
  if(raw.payload?.summary)return raw.payload.summary;if(raw.text)return raw.text;
  if(Array.isArray(raw.message?.content))return raw.message.content.map(c=>c.text||(c.type==='tool_use'?`${c.name}\n${JSON.stringify(c.input)}`:'')).filter(Boolean).join('\n');
  if(raw.part?.text)return raw.part.text;if(raw.item?.command)return raw.item.command;
  return event.text;
}
function event(label,text,cls='',seq) {
  const el=node('article',undefined,'event '+cls);el.append(node('span',undefined,'event-marker'));const body=node('div',undefined,'event-body');const head=node('div',undefined,'event-head');head.append(node('span',label));if(seq!==undefined)head.append(node('span','#'+seq,'event-seq'));body.append(head,node('p',text,'event-text'));el.append(body);return el;
}
function renderTask(data) {
  const t=task(),p=planTask();if(!t||data.task.id!==t.id)return;
  const timeline=[event('Assigned task',p.prompt,'plan')];if(t.feedback)timeline.push(event('You · latest review',t.feedback));
  for(const e of data.events)timeline.push(event(pretty(e.kind),eventText(e),e.kind,e.seq));
  if(t.error)timeline.push(event('Coordinator report',t.error,'error'));
  if(!data.events.length)timeline.push(node('p','No provider events have been reported yet.','empty'));
  const activity=$('activity'),nearBottom=activity.parentElement.scrollHeight-activity.parentElement.scrollTop-activity.parentElement.clientHeight<70;
  activity.replaceChildren(...timeline);if(nearBottom)activity.parentElement.scrollTop=activity.parentElement.scrollHeight;
  $('stdout').textContent=data.stdout||'No output yet.';$('stderr').textContent=data.stderr||'No errors logged.';
  const notes=data.knowledge.filter(k=>k.task===t.id);$('lessons').replaceChildren(...(notes.length?notes.map(k=>{const el=node('article',undefined,'lesson');el.append(node('h3',k.lesson.context),node('span',`${pretty(k.status)} · confidence ${k.lesson.confidence}`,'status-pill'),node('p',k.lesson.response),node('pre',k.lesson.sources.map(s=>s.path+'\n'+s.sha256).join('\n')),node('p','Revalidate: '+k.lesson.revalidate,'small muted'));return el;}):[node('p','No reviewed lessons for this task. Draft observations require explicit promotion.','empty')]));
  const u=data.usage;$('tokens').textContent=u&&typeof u.input_tokens==='number'?`${u.input_tokens.toLocaleString()} in / ${(u.output_tokens||0).toLocaleString()} out`:'Not reported';
  const duration=t.started_ms?(Number(t.finished_ms||Date.now())-Number(t.started_ms))/1000:null;$('duration').textContent=duration===null?'—':`${Math.max(0,duration).toFixed(1)}s`;
}
async function loadTask() {if(!snapshot)return;const index=selected;try{const data=await request({kind:'task',index});if(index===selected)renderTask(data);}catch(e){notice(e.message,true);}}
async function act(action,revision=snapshot?.revision) {
  try{const accepted=await request({kind:'action',revision,action});notice(accepted.message||`${pretty(accepted.kind)} accepted; waiting for the coordinator.`);await poll();return true;}catch(e){notice(e.message,true);await poll();return false;}
}
async function decision(title,description,evidence,needsReason=false,confirm='Confirm') {
  const dialog=$('decision-dialog');$('dialog-title').textContent=title;$('dialog-description').textContent=description;$('dialog-evidence').textContent=evidence;
  $('dialog-reason').value='';$('dialog-reason').hidden=!needsReason;$('dialog-reason-label').hidden=!needsReason;$('dialog-reason').required=needsReason;$('dialog-confirm').textContent=confirm;dialog.returnValue='';dialog.showModal();
  return new Promise(resolve=>dialog.addEventListener('close',()=>resolve(dialog.returnValue==='confirm'?(needsReason?$('dialog-reason').value:true):null),{once:true}));
}
async function briefTeam(){
 try{const b=await request({kind:'briefing'});$('briefing-time').textContent=`${pretty(b.state)} · captured ${new Date(b.observed_at_ms).toLocaleString()} · read-only`;
 $('briefing-sessions').replaceChildren(...b.sessions.map(s=>{const card=node('article',undefined,'briefing-session');card.append(node('h3',`${s.agent} · ${s.task} · ${pretty(s.state)}`),node('p',s.work),node('p',`Session: ${s.session_id||'not established'} · model: ${s.model||'CLI default'} · attempt ${s.attempt}`,'small muted'),node('p',`Process: ${pretty(s.process.state)} · observed ${new Date(s.process.observed_at_ms).toLocaleTimeString()}`,'small muted'));
 if(s.last_activity)card.append(node('p',`Last activity: ${pretty(s.last_activity.kind)} · ${pretty(s.last_activity.freshness)}${s.last_activity.observed_at_ms?' · '+new Date(s.last_activity.observed_at_ms).toLocaleTimeString():''}`,'small muted'),node('p',s.last_activity.text,'briefing-text'));
 const memory=s.memory_report;card.append(node('p',memory.summary?`Memory report (${memory.source}, ${memory.freshness}): ${memory.summary}`:memory.note,'small muted'));
 if(s.restriction)card.append(node('p',s.restriction.available_at?`Available ${new Date(s.restriction.available_at*1000).toLocaleString()}`:'Availability reset unknown','warning'));
 if(s.attention)card.append(node('p',s.attention,'warning'));card.append(node('p','Next: '+s.next_action));return card;}));
 $('briefing-approvals').textContent=b.approvals.length?'Review requests: '+b.approvals.map(a=>`${a.kind} · ${a.task||'integration candidate'} · ${a.status}`).join('; '):'No recorded approval requests.';
 $('briefing-dialog').showModal();}catch(e){notice(e.message,true);}
}
async function loadReview() {
  try{review=await request({kind:'evidence'});$('review-caption').textContent=review.report.candidate?`Candidate ${review.report.candidate.commit.slice(0,12)} · exact captured evidence`:'No candidate is available.';
    $('test-results').replaceChildren(...review.test_output.map(t=>{const el=node('article',undefined,'test-result'+(t.exit_code!==0||t.timed_out?' failed':''));el.append(node('strong',`${t.exit_code===0&&!t.timed_out?'✓':'×'} ${t.task} · exit ${t.exit_code??'unknown'}${t.timed_out?' · timeout':''}`));if(t.stdout||t.stderr)el.append(node('pre',t.stdout+'\n'+t.stderr));return el;}));
    $('diff').replaceChildren(...review.diff.split('\n').map(line=>node('span',line||' ','diff-line'+(line.startsWith('+')?' add':line.startsWith('-')?' remove':line.startsWith('@@')?' hunk':''))));
    if(review.truncated)notice('Diff exceeds the inline review limit. Use the CLI to inspect the complete patch; UI approval is disabled.',true);
    changeTab('review');renderSnapshot();
  }catch(e){notice(e.message,true);}
}
function renderTools() {
  const requests=snapshot.tool_requests||[];const pending=requests.filter(r=>['pending','approved'].includes(r.status));$('tool-count').textContent=pending.length;$('request-count').textContent=pending.length;
  $('tool-requests').replaceChildren(...(requests.length?requests.map(r=>{
    const el=node('article',undefined,'tool-request');el.append(node('strong',`${r.request.tool} · ${pretty(r.status)}`),node('p',r.context.task,'small muted'),node('pre',JSON.stringify(r.request.input,null,2)),node('code',r.id,'sha'));
    const actions=node('div',undefined,'tool-actions');for(const [kind,label] of [['tool_approve','Review & allow'],['tool_reject','Deny'],['tool_refresh','Refresh']]) {
      // A running worker can be paused on a pending request; deciding it is the only live action.
      const live=snapshot.report.state==='running'&&kind!=='tool_refresh';
      const button=node('button',label);button.disabled=!(idle()||live)||(kind==='tool_refresh'?!['pending','approved'].includes(r.status):r.status!=='pending');
      button.onclick=async()=>{const revision=snapshot.revision;const ok=await decision(label+' tool request',kind==='tool_approve'?'Authorize the exact current input once for 300 seconds. It does not authorize merge.':kind==='tool_refresh'?'Revoke the old request and create a fresh pending request. No execution is approved.':'Deny this exact request. Rework or reassignment is needed before retry.',JSON.stringify(r,null,2),false,label);if(ok)await act({kind,sha:r.id},revision);};actions.append(button);
    }el.append(actions);return el;
  }):[node('span','No recorded requests.','small muted')]));
}
function autoQuotaRefresh(){if(!snapshot||busy()||snapshot.report.state==='running')return;const agent=snapshot.report.plan.agents.find(a=>['claude','codex'].includes(a.parser)&&quotaRefreshDue(quotaRefreshed.get(a.id),Date.now()));if(agent)refreshQuota(agent.id);}
async function poll() {
  if(polling)return;polling=true;
  try{const kinds=JSON.stringify([...new Set((snapshot?.report?.plan?.agents||[]).map(a=>a.parser||'custom'))].sort());const next=await request({kind:'snapshot',icons:!snapshot||kinds!==iconKinds});if(next.icons){iconCache=next.icons;iconKinds=JSON.stringify([...new Set((next.report?.plan?.agents||[]).map(a=>a.parser||'custom'))].sort());}snapshot=next;const job=snapshot.job;
    if(job&&job.state!=='running'&&lastJob!==job.id){lastJob=job.id;if(job.state==='failed')notice(job.error||(job.termination_signal?`Action terminated by signal ${job.termination_signal}; previous evidence is retained.`:'Action failed; previous evidence is retained.'),true);else{
      notice(`${pretty(job.kind)} finished. Run: ${pretty(snapshot.report.state)}.`);
      if(job.kind==='approve'&&review&&review.report.candidate?.commit===snapshot.report.candidate?.commit&&review.report.diff_hash===snapshot.report.diff_hash&&JSON.stringify(review.report.tests)===JSON.stringify(snapshot.report.tests))review.revision=snapshot.revision;
      for(const t of snapshot.report.tasks)if(t.feedback&&drafts.get(t.id)===t.feedback){drafts.delete(t.id);if(task()?.id===t.id&&!composing&&$('feedback').value===t.feedback)$('feedback').value='';}
    }}renderSnapshot();setTimeout(autoQuotaRefresh,0);await loadTask();
  }catch(e){$('connection').textContent='Local connection unavailable';notice(e.message,true);}finally{polling=false;}
}
for(const el of document.querySelectorAll('[data-tab]'))el.onclick=()=>el.dataset.tab==='review'?loadReview():changeTab(el.dataset.tab);
$('brief-team').onclick=briefTeam;$('quota-agent').onchange=()=>{quotaAgent=$('quota-agent').value;renderQuota(profile());};$('quota-refresh').onclick=()=>refreshQuota($('quota-agent').value);$('refresh').onclick=poll;$('review-latest').onclick=loadReview;$('open-review').onclick=loadReview;$('open-tools').onclick=()=>{$('tool-requests').scrollIntoView({behavior:'smooth',block:'center'});};
// Input-method composition (e.g. Korean) must finish before text is replaced or sent.
let composing=false;document.addEventListener('compositionstart',()=>{composing=true;});document.addEventListener('compositionend',()=>{composing=false;});
$('feedback').oninput=()=>{if(task())drafts.set(task().id,$('feedback').value);};
$('send-feedback').onclick=async()=>{const feedback=$('feedback').value.trim();if(!feedback){notice('Enter a targeted revision before sending.',true);return;}await act({kind:'revise',task:task().id,feedback});};
$('reassign').onclick=()=>act({kind:'reassign',task:task().id,agent:$('agent-select').value});$('set-model').onclick=()=>act({kind:'model',task:task().id,model:$('model-select').value});
$('schedule').onclick=()=>{const date=new Date($('resume-at').value);if(!Number.isFinite(date.getTime())){notice('Choose a valid local date and time.',true);return;}act({kind:'schedule',task:task().id,at:date.toISOString().replace('.000Z','Z')});};
$('resume').onclick=()=>act({kind:'resume'});$('recover').onclick=async()=>{const revision=snapshot.revision;if(await decision('Recover interrupted run','Recovery refuses any live or unknown recorded process. Review partial evidence before resuming.',snapshot.run_dir))await act({kind:'recover'},revision);};
$('stop').onclick=async()=>{const reason=await decision('Stop the run','Stop all active tasks and tests, retain their evidence, and require explicit resume.',snapshot.report.plan.goal,true,'Stop run');if(reason)await act({kind:'stop',reason});};
$('approve').onclick=async()=>{const packet=review;if(!packet||packet.truncated)return;if(await decision('Approve verified candidate','Approve exactly the diff and test evidence shown. Merge remains a separate action.',`Candidate: ${packet.report.candidate.commit}\nDiff: ${packet.report.diff_hash}\nTests: ${packet.report.tests.length} verified`,false,'Approve'))await act({kind:'approve',sha:packet.report.candidate.commit},packet.revision);};
$('merge').onclick=async()=>{const packet=review;if(!packet)return;if(await decision('Merge approved candidate','Integrate this exact approved candidate into the target branch. The backend rechecks current Git and test evidence.',`Branch: ${packet.report.branch}\nCandidate: ${packet.report.candidate.commit}`,false,'Merge'))await act({kind:'merge',sha:packet.report.candidate.commit},packet.revision);};
$('reject').onclick=async()=>{const revision=snapshot.revision;const reason=await decision('Reject candidate','Reject the current integration candidate. Send targeted feedback to prepare a new revision.',snapshot.report.candidate?.commit||'No candidate',true,'Reject');if(reason)await act({kind:'reject',reason},revision);};
const roomShortcut=e=>{if((e.metaKey||e.ctrlKey)&&e.key==='Enter'&&!e.isComposing&&!composing&&!$('decision-dialog').open){e.preventDefault();if(!$('send-feedback').disabled)$('send-feedback').click();}};
document.addEventListener('keydown',roomShortcut);
// DD-71: 1 s while the run or an action is active, 5 s when idle, paused while hidden.
let roomTimer=null,roomStopped=false;function roomSchedule(){clearTimeout(roomTimer);if(roomStopped||document.hidden)return;const fast=snapshot?.job?.state==='running'||['running','ready'].includes(snapshot?.report?.state);roomTimer=setTimeout(async()=>{await poll();roomSchedule();},fast?1000:5000);}
const roomVisibility=()=>{if(document.hidden)clearTimeout(roomTimer);else if(!roomStopped)poll().then(roomSchedule);};document.addEventListener('visibilitychange',roomVisibility);
poll().then(roomSchedule);window.stopRoom=()=>{roomStopped=true;clearTimeout(roomTimer);document.removeEventListener('visibilitychange',roomVisibility);document.removeEventListener('keydown',roomShortcut);};
