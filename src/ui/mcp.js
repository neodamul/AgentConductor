// DD-68: on-demand configuration; DD-138 coordinator-selected document reads.
const REFERENCE_LIMIT=5;
function mcpUi({q,el,request,action,save,mark,capture,poll,state,draft,notice,showStep,stale=()=>'',onReferences=()=>{}}) {
 let sourceReferences=[],sourceReady=false;
 let shown=null,review=null,suggestionsStamp=null,sourceCount=0,pendingDiscovery=null,previewing=false,failedShown=null,goalStamp=null,goalRangeStamp=null;const catalogs=new Map(),notes=new Map();
 // Goal references (jira:KEY, confluence:ID) are drawn as tags behind the editable text, so they
 // stay plain text to type or delete; hovering one shows its title and summary once read.
 function rememberNotes(p){for(const r of p.records||[]){const key=r.reference.kind+':'+r.reference.id,text=p.summaries?.[key]?.text||r.body||'';notes.set(key,{title:r.title||'',summary:text.slice(0,600)});}}
 function alignGoalTags(){const g=q('goal'),box=q('goal-tags');if(!g?.getBoundingClientRect||!box?.style)return;const r=g.getBoundingClientRect(),p=g.parentElement.getBoundingClientRect();Object.assign(box.style,{inset:'auto',left:r.left-p.left+'px',top:r.top-p.top+'px',width:r.width+'px',height:r.height+'px'});}
 if(typeof ResizeObserver==='function'&&q('goal'))new ResizeObserver(alignGoalTags).observe(q('goal'));
 function goalTags(text,ranges=[]){const g=q('goal'),box=q('goal-tags');if(typeof box?.replaceChildren!=='function')return;alignGoalTags();const rangeStamp=JSON.stringify(ranges);if(text===goalStamp&&rangeStamp===goalRangeStamp)return;goalStamp=text;goalRangeStamp=rangeStamp;
  // The tag layer copies the textarea's actual text metrics so tags sit exactly under their text.
  if(typeof getComputedStyle==='function'&&g){const cs=getComputedStyle(g);for(const p of ['fontFamily','fontSize','fontWeight','fontStyle','lineHeight','letterSpacing','wordSpacing','tabSize','textIndent','paddingTop','paddingRight','paddingBottom','paddingLeft','borderTopWidth','borderRightWidth','borderBottomWidth','borderLeftWidth'])box.style[p]=cs[p];}const parts=[];let last=0;for(const range of ranges){if(!Number.isInteger(range.start)||!Number.isInteger(range.end)||range.start<last||range.end<=range.start||range.end>text.length)continue;parts.push(document.createTextNode(text.slice(last,range.start)));const tag=document.createElement('mark');tag.className='goal-tag';tag.dataset.ref=range.reference;tag.textContent=text.slice(range.start,range.end);parts.push(tag);last=range.end;}parts.push(document.createTextNode(text.slice(last)+'\n'));box.replaceChildren(...parts);box.scrollTop=g.scrollTop;}
 function tagAt(x,y){for(const tag of q('goal-tags').querySelectorAll('.goal-tag'))for(const r of tag.getClientRects())if(x>=r.left&&x<=r.right&&y>=r.top&&y<=r.bottom)return tag.dataset.ref;return null;}
 function tagTitle(ref){const n=notes.get(ref)||state()?.source_notes?.[ref];return n?(n.title?ref+' · '+n.title:ref)+(n.summary?'\n\n'+n.summary:''):acMessage('mcp.reference_not_read',{value1:ref});}
 if(typeof q('goal')?.addEventListener==='function'){q('goal').addEventListener('input',()=>updateSourceSuggestions(q('goal').value));q('goal').addEventListener('scroll',()=>{q('goal-tags').scrollTop=q('goal').scrollTop;});
  q('goal').addEventListener('mousemove',e=>{const ref=tagAt(e.clientX,e.clientY),title=ref?tagTitle(ref):'';if(q('goal').title!==title)q('goal').title=title;});}
 let editingName=null,selectedServer='';const discoveryErrors=new Map();
 const configJson=()=>{const c=state()?.config||{};return JSON.stringify({mcp:c.mcp||{},goal_sources:c.goal_sources||{}},null,2);};
 const catalogFor=name=>{const config=state()?.config.mcp?.[name],stored=catalogs.get(name);return stored&&stored.config===JSON.stringify(config)?stored:null;};
 function toggleView(edit){q('mcp-list-view').hidden=edit;q('mcp-edit-view').hidden=!edit;q('mcp-close').hidden=edit;q('mcp-edit-cancel').hidden=!edit;q('mcp-edit-save').hidden=!edit;q('mcp-title').textContent=edit?acMessage(editingName?'mcp.edit_server':'mcp.add_server'):acMessage('mcp.manage_mcp_servers');}
 function mkButton(key,run){const b=el('button',()=>acMessage(key),'quiet small-button');b.type='button';b.onclick=run;return b;}
 function renderServerList(){const entries=Object.entries(state()?.config.mcp||{});AC_I18N.replace(q('mcp-servers'),...(entries.length?entries.map(([name,config])=>{const row=el('div',undefined,'mcp-server');row.setAttribute('role','listitem');const info=el('div'),title=el('strong',name),meta=el('small',config.type==='remote'?`${acMessage('mcp.remote_url')} · ${config.url}`:`${acMessage('mcp.local_command')} · ${(config.command||[])[0]||''}`),status=el('span',undefined,'mcp-status');let stateName='unknown';if(pendingDiscovery?.server===name)stateName='checking';else if(discoveryErrors.has(name))stateName='error';else if(catalogFor(name))stateName='ok';status.classList.add(stateName);status.textContent=stateName==='checking'?acMessage('mcp.checking'):stateName==='ok'?acMessage('mcp.tools_found',{value1:catalogFor(name).tools.length}):stateName==='error'?acMessage('mcp.discovery_failed',{value1:discoveryErrors.get(name)}):acMessage('mcp.not_checked');status.setAttribute('role','status');info.append(title,meta,status);const actions=el('div',undefined,'mcp-server-actions');actions.append(mkButton('mcp.edit',()=>openEditor(name)),mkButton('mcp.rediscover',()=>discoverServer(name)),mkButton('mcp.remove_server',()=>removeServer(name)));row.append(info,actions);return row;}):[el('p',()=>acMessage('mcp.no_servers_registered'),'mcp-empty')]));}
 function renderCatalog(){const catalog=catalogFor(selectedServer);AC_I18N.bind(q('mcp-catalog-status'),()=>catalog?acMessage('mcp.tools_found_for',{value1:catalog.tools.length,value2:selectedServer}):pendingDiscovery?.server===selectedServer?acMessage('mcp.loading_tools'):discoveryErrors.has(selectedServer)?acMessage('mcp.lookup_failed',{value1:discoveryErrors.get(selectedServer)}):selectedServer?acMessage('mcp.catalog_empty_for_server',{value1:selectedServer}):'');}
 function render(){renderServerList();renderCatalog();}
 function openSettings(){q('mcp-json').value=configJson();editingName=null;selectedServer=Object.keys(state()?.config.mcp||{})[0]||'';toggleView(false);render();q('settings').close();requestAnimationFrame(()=>{if(!q('mcp-dialog').open)q('mcp-dialog').showModal();});}
 function addSecret(key='',value=''){const row=el('div',undefined,'mcp-secret'),name=el('input'),secret=el('input');name.value=key;secret.value=value;name.placeholder=q('mcp-kind').value==='remote'?'Authorization':'TOKEN';name.setAttribute('aria-label',acMessage('mcp.name'));secret.placeholder='{env:ATLASSIAN_TOKEN}';secret.setAttribute('aria-label',acMessage('mcp.value'));secret.autocomplete='off';const remove=mkButton('mcp.remove_row',()=>row.remove());row.append(name,secret,remove);q('mcp-secrets').append(row);}
 function syncKind(){const remote=q('mcp-kind').value==='remote';q('mcp-url-label').hidden=!remote;q('mcp-command-label').hidden=remote;q('mcp-url').required=remote;q('mcp-executable').required=!remote;q('mcp-secret-title').textContent=acMessage(remote?'mcp.headers':'mcp.environment_variables');}
 function openEditor(name=null){editingName=name;const server=name?state().config.mcp[name]:null,local=server?.type==='local';q('mcp-name').value=name||'';q('mcp-kind').value=server?.type||'remote';q('mcp-url').value=server?.url||'';q('mcp-executable').value=local?server.command?.[0]||'':'';q('mcp-arguments').value=local?(server.command||[]).slice(1).join('\n'):'';AC_I18N.replace(q('mcp-secrets'));const pairs=server?(local?server.environment:server.headers)||{}:{};for(const[key,value]of Object.entries(pairs))addSecret(key,value);if(!Object.keys(pairs).length)addSecret();q('mcp-edit-error').textContent='';selectedServer=name||'';syncKind();toggleView(true);q('mcp-name').focus();}
 function secretValues(){return Object.fromEntries([...q('mcp-secrets').querySelectorAll('.mcp-secret')].map(row=>[row.children[0].value.trim(),row.children[1].value.trim()]).filter(([key,value])=>key&&value));}
 async function discoverServer(name){const config=state()?.config.mcp?.[name];if(!config||config.enabled===false)return;selectedServer=name;discoveryErrors.delete(name);pendingDiscovery={server:name,config:JSON.stringify(config)};render();if(!await action({kind:'mcp_discover',server:name})){pendingDiscovery=null;render();}}
 async function saveServer(){const name=q('mcp-name').value.trim(),kind=q('mcp-kind').value,config=state().config,oldName=editingName,old=oldName?config.mcp[oldName]:null;if(!name){q('mcp-edit-error').textContent=acMessage('mcp.enter_server_name');return;}if(name!==oldName&&config.mcp[name]){q('mcp-edit-error').textContent=acMessage('mcp.server_name_exists',{value1:name});return;}const next=kind==='remote'?{...(old?.type==='remote'?old:{}),type:'remote',url:q('mcp-url').value.trim(),headers:secretValues()}:{...(old?.type==='local'?old:{}),type:'local',command:[q('mcp-executable').value.trim(),...q('mcp-arguments').value.split('\n').map(value=>value.trim()).filter(Boolean)],environment:secretValues()};if(kind==='remote'&&!next.url){q('mcp-edit-error').textContent=acMessage('mcp.enter_server_url');return;}if(kind==='local'&&!next.command[0]){q('mcp-edit-error').textContent=acMessage('mcp.enter_executable');return;}if(kind==='remote'&&!Object.keys(next.headers).length)delete next.headers;if(kind==='local'&&!Object.keys(next.environment).length)delete next.environment;const changed=!old||JSON.stringify(old)!==JSON.stringify(next),renamed=!!oldName&&oldName!==name,mcp={...config.mcp},sources={...(config.goal_sources||{})};if(renamed){delete mcp[oldName];for(const source of Object.values(sources))if(source.server===oldName)source.server=name;catalogs.delete(oldName);discoveryErrors.delete(oldName);}mcp[name]=next;if(changed)for(const type of ['jira','confluence'])if(sources[type]?.server===name)delete sources[type];if(await action({kind:'mcp_save',settings:JSON.stringify({mcp,goal_sources:sources})})){draft().mcp=state().config.mcp;draft().goal_sources=state().config.goal_sources;selectedServer=name;editingName=name;q('mcp-json').value=configJson();toggleView(false);render();if(changed||renamed){catalogs.delete(name);discoveryErrors.delete(name);await discoverServer(name);}}}
 async function removeServer(name){const config=state().config,mcp={...config.mcp};delete mcp[name];const goal_sources=Object.fromEntries(Object.entries(config.goal_sources||{}).filter(([,source])=>source.server!==name));if(await action({kind:'mcp_save',settings:JSON.stringify({mcp,goal_sources})})){catalogs.delete(name);discoveryErrors.delete(name);selectedServer=Object.keys(mcp)[0]||'';draft().mcp=state().config.mcp;draft().goal_sources=state().config.goal_sources;render();q('mcp-json').value=configJson();}}
 q('mcp-settings').onclick=openSettings;q('mcp-add').onclick=()=>openEditor();q('mcp-kind').onchange=syncKind;q('mcp-secret-add').onclick=()=>addSecret();q('mcp-close').onclick=()=>q('mcp-dialog').close();q('mcp-edit-cancel').onclick=()=>{toggleView(false);render();};q('mcp-form').onsubmit=event=>{event.preventDefault();return saveServer();};
 q('mcp-export').onclick=()=>{q('mcp-json').value=configJson();q('mcp-json').focus();q('mcp-json').select();};
 q('mcp-import').onclick=async()=>{if(await action({kind:'mcp_save',settings:q('mcp-json').value})){catalogs.clear();discoveryErrors.clear();draft().mcp=state().config.mcp;draft().goal_sources=state().config.goal_sources;render();notice(()=>acMessage('mcp.imported_settings'));}};
 q('mcp-preview').onclick=async()=>{capture();const c=draft();if(!c.coordinator||!c.profiles?.some(p=>p.id===c.coordinator)){notice(()=>acMessage('mcp.coordinator_required'),true);showStep?.('workspace',true);q('coordinator').focus?.();return;}if(!await save())return;
  // The review window opens at once and shows the lookup in progress; records fill in when it ends.
  review=null;previewing=true;lastProgress=null;AC_I18N.replace(q('mcp-records'));AC_I18N.bind(q('mcp-meta'),()=>'');q('mcp-guide').open=true;q('mcp-retry').hidden=true;q('mcp-failures').hidden=true;q('mcp-apply').disabled=true;q('mcp-loading').hidden=false;if(!q('mcp-review').open)q('mcp-review').showModal();
  if(!await action({kind:'mcp_preview'})){previewing=false;q('mcp-loading').hidden=true;q('mcp-review').close();}};
 // GitHub #903: only the documents the last preview failed to read are fetched again.
 q('mcp-retry').onclick=async()=>{const kept=review;review=null;previewing=true;lastProgress=null;q('mcp-retry').hidden=true;q('mcp-apply').disabled=true;if(!await action({kind:'mcp_preview',retry:true})){previewing=false;showPreview(kept);}};
 q('reference').onclick=()=>{q('source-id').value='';q('source-reference').showModal();};
 q('source-add').onclick=()=>{const kind=q('source-kind').value,id=q('source-id').value.trim();if(sourceCount>=REFERENCE_LIMIT&&!sourceReferences.includes(kind+':'+id)){notice(()=>acMessage('mcp.reference_limit_reached',{limit:REFERENCE_LIMIT}),true);return;}const valid=kind==='jira'?/^[A-Z][A-Z0-9_]*-\d+$/.test(id):/^\d{1,32}$/.test(id);if(!valid){notice(()=>(acMessage("mcp.enter_a_jira_issue_key_or_numeric_confluence")),true);return;}q('goal').value=q('goal').value.trimEnd()+` [${kind==='jira'?'Jira':'Confluence'}:${id}]`;capture();mark();q('source-reference').close();};
 q('mcp-apply').onclick=async()=>{if(!review)return;const selected=[...q('mcp-records').querySelectorAll('input:checked')].map(n=>n.value);const r=await action({kind:'mcp_apply',preview_id:review.id,selected});if(r){q('mcp-review').close();notice(()=>(acMessage("mcp.selected_source_evidence_approved_locally_generate_a_plan")));}};
 // GitHub #903: one card per Goal reference with its step, elapsed time and live log (same structure as the approved mock).
 const sourceT=(key,vars)=>acMessage(`mcp.${key}`,vars);
 function sourceDuration(ms){const s=Math.max(0,Math.round(ms/1000));return s<60?sourceT('duration_seconds',{s}):sourceT('duration_minutes',{m:Math.floor(s/60),s:String(s%60).padStart(2,'0')});}
 function sourceSize(bytes){return bytes<1024?bytes+' B':Math.round(bytes/1024)+' KB';}
 function sourceFailedStep(doc){const events=doc.log.map(e=>e.event);return events.includes('summary')?2:events.includes('mcp_call')?1:0;}
 function sourceCard(doc,{now,record,summary,review}){
  const mk=(tag,cls,text)=>{const n=document.createElement(tag);if(cls)n.className=cls;if(text!==undefined)n.textContent=text;return n;};
  const running=['selection','reading','summary'].includes(doc.stage),failed=doc.stage==='failed';
  const card=mk('section','source-doc'+(running?' running':'')+(failed?' failed':''));card.dataset.reference=doc.reference;card.dataset.stage=doc.stage;
  const head=mk('div','source-doc-head');
  if(review){const pick=mk('input');pick.type='checkbox';pick.value=doc.reference;pick.checked=!!record;pick.disabled=!record||doc.reference.startsWith('jira:');head.append(pick);}
  const [kind,id]=doc.reference.split(':');
  head.append(mk('span','source-ref',(kind==='jira'?'Jira':'Confluence')+':'+id));
  const title=doc.title||record?.title;head.append(mk('strong','source-title'+(title?'':' pending'),title||acMessage(failed?'mcp.title_not_supplied':'mcp.title_pending')));
  const label=failed?'stage_failed':doc.stage==='done'?(review?'stage_review':'stage_done'):'stage_'+doc.stage;
  head.append(mk('span','source-state '+(failed?'failed':doc.stage==='waiting'?'waiting':['done','summary_wait'].includes(doc.stage)?'ready':'running'),sourceT(label)));
  head.append(mk('span','source-time',doc.started_ms?sourceDuration((doc.ended_ms||now)-doc.started_ms):'—'));
  card.append(head);
  const current={waiting:-1,selection:0,reading:1,summary_wait:2,summary:2,done:3}[doc.stage]??sourceFailedStep(doc);
  const steps=mk('ol','source-steps');['step_select','step_read','step_summary','step_review'].forEach((key,i)=>steps.append(mk('li','source-step'+(failed?(i<current?' done':i===current?' failed':''):(i<current?' done':i===current?' current':'')),sourceT(key))));
  card.append(steps);
  if(running||failed){const lines=(doc.log||[]).map(e=>{const at=Math.max(0,Math.round((e.at_ms-doc.started_ms)/1000));return String(Math.floor(at/60)).padStart(2,'0')+':'+String(at%60).padStart(2,'0')+' '+sourceT('log_'+e.event,{detail:e.event==='read_done'?sourceSize(Number(e.detail)):e.detail});});if(lines.length)card.append(mk('div','source-log',lines.join('\n')));}
  if(failed&&doc.error)card.append(mk('p','source-error',doc.error));
  if(record){
   if(summary)card.append(mk('div','source-summary',summary.text));
   const full=mk('details','source-full');full.append(mk('summary','',sourceT('full_source_size',{size:sourceSize(record.body.length)})));full.ontoggle=()=>{if(full.open&&full.childNodes.length===1)full.append(mk('pre','source-body',record.body));};card.append(full);
   card.append(mk('p','source-provenance',sourceT('provenance',{server:record.server,tool:record.tool,version:record.version||'—',sha:record.sha256.slice(0,8)})));
  }
  return card;
 }
 function sourceMeta(progress,finished,now){const docs=progress.documents,done=docs.filter(d=>d.stage==='done'||!finished&&d.stage==='summary_wait').length,failed=docs.filter(d=>d.stage==='failed').length,end=finished?Math.max(progress.started_ms,...docs.map(d=>d.ended_ms||progress.started_ms)):now;const elapsed=sourceDuration(end-progress.started_ms);return finished?sourceT('meta_done',{done,failed,elapsed}):sourceT('meta_running',{total:docs.length,done,elapsed});}
 const runningDoc=progress=>progress?.documents?.find(d=>['selection','reading','summary'].includes(d.stage));
 function sourceStatus(progress,now){const doc=runningDoc(progress);return doc?sourceT('status_running',{done:progress.documents.filter(d=>d.stage==='done'||d.stage==='summary_wait').length,total:progress.documents.length,reference:doc.reference,stage:sourceT('stage_'+doc.stage),elapsed:sourceDuration(now-progress.started_ms)}):'';}
 let lastProgress=null;
 function renderProgress(progress){if(!progress?.documents?.length||review)return;lastProgress=progress;const now=Date.now();q('mcp-loading').hidden=true;q('mcp-guide').open=true;AC_I18N.replace(q('mcp-records'),...progress.documents.map(doc=>sourceCard(doc,{now,review:false})));AC_I18N.bind(q('mcp-meta'),()=>sourceMeta(progress,false,now));q('mcp-retry').hidden=true;}
 function showPreview(p){review=p;const records=new Map(p.records.map(r=>[r.reference.kind+':'+r.reference.id,r])),failures=new Map(p.failures.map(f=>[f.reference.kind+':'+f.reference.id,f.error]));
  // Progress from before #903, or a missing progress file, falls back to the result itself.
  const known=lastProgress?.documents?.length&&lastProgress.documents.every(d=>records.has(d.reference)||failures.has(d.reference))?lastProgress:{started_ms:p.fetched_at*1000,documents:[...records.keys(),...failures.keys()].map(reference=>({reference,stage:records.has(reference)?'done':'failed',log:[]}))};
  const docs=known.documents.map(d=>records.has(d.reference)?{...d,stage:'done'}:{...d,stage:'failed',error:d.error||failures.get(d.reference)});const progress={...known,documents:docs},now=Date.now();
  AC_I18N.replace(q('mcp-records'),...docs.map(doc=>sourceCard(doc,{now,record:records.get(doc.reference),summary:p.summaries?.[doc.reference],review:true})));
  AC_I18N.bind(q('mcp-meta'),()=>sourceMeta(progress,true,now));q('mcp-guide').open=false;q('mcp-failures').hidden=true;q('mcp-retry').hidden=!p.failures.length;
  q('mcp-apply').disabled=!p.records.length||p.failures.some(f=>f.reference.kind==='jira');previewing=false;lastProgress=null;q('mcp-loading').hidden=true;rememberNotes(p);if(!q('mcp-review').open)q('mcp-review').showModal();}
 function renderSuggestions(suggestions){AC_I18N.replace(q('source-suggestions'),...suggestions.map(ref=>{const b=el('button',()=>acMessage('mcp.use',{value1:ref}),'quiet small-button');b.type='button';b.onclick=()=>{const[k,id]=ref.split(':');q('source-kind').value=k;q('source-id').value=id;q('source-reference').showModal();};return b;}));q('source-suggestions').hidden=!suggestions.length;}
 function updateSourceSuggestions(goal){
  suggestionsStamp=goal;sourceCount=0;sourceReferences=[];sourceReady=false;
  goalTags(goal,[]);
  if(!goal.trim()){sourceReady=true;renderSuggestions([]);onReferences();return;}
  request({kind:'mcp_source_suggestions',goal}).then(result=>{
   if(suggestionsStamp!==goal)return;
   sourceReferences=Array.isArray(result.references)?result.references:[];sourceCount=sourceReferences.length;sourceReady=true;
   goalTags(goal,Array.isArray(result.reference_ranges)?result.reference_ranges:[]);
   renderSuggestions(Array.isArray(result.suggestions)?result.suggestions:[]);
   // GitHub #279: the preview button and source line follow the answer at once, not at the next refresh.
   update();onReferences();
  }).catch(()=>{if(suggestionsStamp===goal)renderSuggestions([]);});
 }
 function update(){
  const s=state();if(!s)return;const job=s.job,goal=draft()?.goal??s.config.goal;
  // An MCP lookup in progress is visible beside the source buttons and in the open windows.
  const looking=job?.kind==='mcp'&&job.state==='running';q('source-status').classList?.toggle('mcp-busy',looking);
  // GitHub #279: the lookup counts from its start and says when the app sends no new state; the spinner alone keeps turning.
  const late=looking?stale():'',since=looking?' '+AC_ELAPSED(job.started_ms||Date.now())+(late?' · '+late:''):'';q('mcp-loading-time').textContent=since;
  if(looking&&pendingDiscovery)AC_I18N.bind(q('mcp-catalog-status'),()=>acMessage('mcp.loading_tools')+since);
  if(job?.kind==='mcp'&&['failed','interrupted','stopped'].includes(job.state)&&job.id!==failedShown&&(previewing||pendingDiscovery)){
   failedShown=job.id;const error=String(job.error||'').replace(/^agentconductor:\s*/,'').trim();
   if(previewing){previewing=false;q('mcp-loading').hidden=true;AC_I18N.bind(q('mcp-failures'),()=>acMessage('mcp.lookup_failed',{value1:error||job.state}));q('mcp-failures').hidden=false;}
   else{discoveryErrors.set(pendingDiscovery.server,error||job.state);pendingDiscovery=null;render();}
  }
  if(goal!==suggestionsStamp)updateSourceSuggestions(goal);
  // GitHub #907: five references at most; adding stops at the limit and a longer Goal cannot be previewed.
  const over=sourceCount>REFERENCE_LIMIT,full=sourceCount>=REFERENCE_LIMIT;
  q('mcp-preview').disabled=job?.state==='running'||!sourceCount||over;
  {const b=q('reference');b.disabled=full;b.title=full?acMessage('mcp.reference_limit_reached',{limit:REFERENCE_LIMIT}):'';}
  for(const b of q('source-suggestions').querySelectorAll?.('button')||[])b.disabled=full;
  q('source-status').classList?.toggle('over',over&&!looking);
  // A lookup still running after the workspace reopened keeps its progress, so its window can show the result.
  if(looking&&s.mcp_progress?.documents){previewing=true;renderProgress(s.mcp_progress);}
  AC_I18N.bind(q('source-status'),()=>looking&&runningDoc(s.mcp_progress)?sourceStatus(s.mcp_progress,Date.now()):looking?acMessage('mcp.lookup_running')+since:over?acMessage('mcp.too_many_references',{limit:REFERENCE_LIMIT,count:sourceCount}):s.imports?acMessage('mcp.applied_sources',{value1:s.imports.selected.length}):sourceCount?acMessage('mcp.references_count',{count:sourceCount,limit:REFERENCE_LIMIT}):'');
  q('mcp-settings').disabled=job?.state==='running';
  // GitHub #893: stored discoveries come back for every server whose config is unchanged, after reopen or restart.
  if(s.mcp_catalogs){let added=false;for(const[server,result]of Object.entries(s.mcp_catalogs)){const config=state().config.mcp?.[server];if(!config||catalogFor(server)||pendingDiscovery?.server===server)continue;catalogs.set(server,{tools:result.tools,config:JSON.stringify(config)});added=true;}if(added&&q('mcp-dialog').open)render();}
  if(job?.kind==='mcp'&&job.state==='completed'&&job.id!==shown&&s.mcp_result){
   shown=job.id;const result=s.mcp_result;
   if(result.tools){const server=result.server,config=state().config.mcp[server];if(config&&(!pendingDiscovery||pendingDiscovery.server===server&&pendingDiscovery.config===JSON.stringify(config))){catalogs.set(server,{tools:result.tools,config:JSON.stringify(config)});discoveryErrors.delete(server);}pendingDiscovery=null;render();}
   else showPreview(result);
  }
 }
 return {update,referenceState:()=>({goal:suggestionsStamp,references:sourceReferences,ready:sourceReady}),wantsResult:job=>q('mcp-dialog').open||job?.kind==='mcp'&&job.state==='completed'&&job.id!==shown};
}
