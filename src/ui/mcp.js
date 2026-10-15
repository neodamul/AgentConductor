// DD-68: on-demand configuration; DD-138 coordinator-selected document reads.
function mcpUi({q,el,request,action,save,mark,capture,poll,state,draft,notice,showStep}) {
 let shown=null,review=null,suggestionsStamp=null,sourceCount=0,pendingDiscovery=null,previewing=false,failedShown=null,goalStamp=null,goalRangeStamp=null;const catalogs=new Map(),notes=new Map();
 // Goal references (jira:KEY, confluence:ID) are drawn as tags behind the editable text, so they
 // stay plain text to type or delete; hovering one shows its title and summary once read.
 function rememberNotes(p){for(const r of p.records||[]){const key=r.reference.kind+':'+r.reference.id,text=p.summaries?.[key]?.text||r.body||'';notes.set(key,{title:r.title||'',summary:text.slice(0,600)});}}
 function goalTags(text,ranges=[]){const g=q('goal'),box=q('goal-tags');if(typeof box?.replaceChildren!=='function')return;const rangeStamp=JSON.stringify(ranges);if(text===goalStamp&&rangeStamp===goalRangeStamp)return;goalStamp=text;goalRangeStamp=rangeStamp;
  // The tag layer copies the textarea's actual text metrics so tags sit exactly under their text.
  if(typeof getComputedStyle==='function'&&g){const cs=getComputedStyle(g);for(const p of ['fontFamily','fontSize','fontWeight','fontStyle','lineHeight','letterSpacing','wordSpacing','tabSize','textIndent','paddingTop','paddingRight','paddingBottom','paddingLeft','borderTopWidth','borderRightWidth','borderBottomWidth','borderLeftWidth'])box.style[p]=cs[p];}const parts=[];let last=0;for(const range of ranges){if(!Number.isInteger(range.start)||!Number.isInteger(range.end)||range.start<last||range.end<=range.start||range.end>text.length)continue;parts.push(document.createTextNode(text.slice(last,range.start)));const tag=document.createElement('mark');tag.className='goal-tag';tag.dataset.ref=range.reference;tag.textContent=range.reference;parts.push(tag);last=range.end;}parts.push(document.createTextNode(text.slice(last)+'\n'));box.replaceChildren(...parts);box.scrollTop=g.scrollTop;}
 function tagAt(x,y){for(const tag of q('goal-tags').querySelectorAll('.goal-tag'))for(const r of tag.getClientRects())if(x>=r.left&&x<=r.right&&y>=r.top&&y<=r.bottom)return tag.dataset.ref;return null;}
 function tagTitle(ref){const n=notes.get(ref)||state()?.source_notes?.[ref];return n?(n.title?ref+' · '+n.title:ref)+(n.summary?'\n\n'+n.summary:''):acMessage('mcp.reference_not_read',{value1:ref});}
 if(typeof q('goal')?.addEventListener==='function'){q('goal').addEventListener('input',()=>updateSourceSuggestions(q('goal').value));q('goal').addEventListener('scroll',()=>{q('goal-tags').scrollTop=q('goal').scrollTop;});
  q('goal').addEventListener('mousemove',e=>{const ref=tagAt(e.clientX,e.clientY),title=ref?tagTitle(ref):'';if(q('goal').title!==title)q('goal').title=title;});}
 const example={mcp:{company:{type:'local',command:['company-mcp'],enabled:true,environment:{TOKEN:'{env:COMPANY_MCP_TOKEN}'}}}};
 function servers(){const current=q('mcp-server').value,names=Object.keys(state()?.config.mcp||{});AC_I18N.replace(q('mcp-server'),...names.map(name=>{const o=el('option',name);o.value=name;return o;}));if(names.includes(current))q('mcp-server').value=current;q('mcp-discover').disabled=!names.length;}
 function renderCatalog(){const server=q('mcp-server').value,config=state()?.config.mcp?.[server],stored=catalogs.get(server),catalog=stored?.config===JSON.stringify(config)?stored:null;q('mcp-server-json').value=config?JSON.stringify(config,null,2):'';q('mcp-server-save').disabled=!config;q('mcp-discover').disabled=!config||config.enabled===false||state()?.job?.state==='running';AC_I18N.text(q('mcp-tools'),catalog?JSON.stringify({server,tools:catalog.tools},null,2):()=>acMessage('mcp.catalog_empty'));}
 function openSettings(){const c=state()?.config;q('mcp-json').value=JSON.stringify(Object.keys(c?.mcp||{}).length?{mcp:c.mcp,goal_sources:c.goal_sources||{}}:example,null,2);servers();renderCatalog();q('settings').close();requestAnimationFrame(()=>{if(!q('mcp-dialog').open)q('mcp-dialog').showModal();});}
 q('mcp-settings').onclick=openSettings;q('mcp-server').onchange=renderCatalog;
 function saved(){draft().mcp=state().config.mcp;draft().goal_sources=state().config.goal_sources;servers();renderCatalog();notice(()=>acMessage('mcp.mcp_definitions_saved_no_server_was_started'));}
 q('mcp-save').onclick=async()=>{if(await action({kind:'mcp_save',settings:q('mcp-json').value}))saved();};
 q('mcp-server-save').onclick=async()=>{const name=q('mcp-server').value;if(!name)return;try{const config=JSON.parse(q('mcp-server-json').value),c=state().config;if(await action({kind:'mcp_save',settings:JSON.stringify({mcp:{...c.mcp,[name]:config},goal_sources:c.goal_sources||{}})})){q('mcp-json').value=JSON.stringify({mcp:state().config.mcp,goal_sources:state().config.goal_sources||{}},null,2);saved();}}catch(e){notice(e.message,true);}};
 q('mcp-discover').onclick=async()=>{const server=q('mcp-server').value;pendingDiscovery={server,config:JSON.stringify(state().config.mcp[server])};AC_I18N.text(q('mcp-tools'),()=>acMessage('mcp.loading_tools'));if(!await action({kind:'mcp_discover',server})){pendingDiscovery=null;renderCatalog();}};
 q('mcp-preview').onclick=async()=>{capture();const c=draft();if(!c.coordinator||!c.profiles?.some(p=>p.id===c.coordinator)){notice(()=>acMessage('mcp.coordinator_required'),true);showStep?.('workspace',true);q('coordinator').focus?.();return;}if(!await save())return;
  // The review window opens at once and shows the lookup in progress; records fill in when it ends.
  review=null;previewing=true;AC_I18N.replace(q('mcp-records'));q('mcp-failures').hidden=true;q('mcp-apply').disabled=true;q('mcp-loading').hidden=false;if(!q('mcp-review').open)q('mcp-review').showModal();
  if(!await action({kind:'mcp_preview'})){previewing=false;q('mcp-loading').hidden=true;q('mcp-review').close();}};
 for(const kind of ['jira','confluence'])q(kind+'-reference').onclick=()=>{q('source-kind').value=kind;q('source-id').value='';q('source-reference').showModal();};
 q('source-add').onclick=()=>{const kind=q('source-kind').value,id=q('source-id').value.trim();const valid=kind==='jira'?/^[A-Z][A-Z0-9_]*-\d+$/.test(id):/^\d{1,32}$/.test(id);if(!valid){notice(()=>(acMessage("mcp.enter_a_jira_issue_key_or_numeric_confluence")),true);return;}q('goal').value=q('goal').value.trimEnd()+` ${kind}:${id}`;capture();mark();q('source-reference').close();};
 q('mcp-apply').onclick=async()=>{if(!review)return;const selected=[...q('mcp-records').querySelectorAll('input:checked')].map(n=>n.value);const r=await action({kind:'mcp_apply',preview_id:review.id,selected});if(r){q('mcp-review').close();notice(()=>(acMessage("mcp.selected_source_evidence_approved_locally_generate_a_plan")));}};
 function showPreview(p){review=p;AC_I18N.replace(q('mcp-records'),...p.records.map(r=>{const section=el('section',undefined,'source-record'),label=el('label',undefined,'startup-check'),input=el('input');input.type='checkbox';input.checked=true;input.disabled=r.reference.kind==='jira';input.value=r.reference.kind+':'+r.reference.id;label.append(input,el('strong',input.value));const source=el('pre',()=>acMessage('mcp.version_updated_retrieved_sha_256',{value1:r.server,value2:r.tool,value3:r.title||acMessage('mcp.title_not_supplied'),value4:r.url||acMessage('mcp.url_not_supplied'),value5:r.version||acMessage('mcp.not_supplied'),value6:r.updated_at||acMessage('mcp.not_supplied'),value7:new Date(p.fetched_at*1000).toLocaleString(),value8:r.sha256}),'small muted'),summary=p.summaries?.[input.value];section.append(label,source);if(summary){section.append(el('p',()=>acMessage('mcp.source_summary',{path:state().root+'/mcp/'+summary.markdown_file}),'small source-path'),el('pre',summary.text,'source-body'));const detail=el('details');detail.append(el('summary',()=>acMessage('mcp.full_source')));detail.ontoggle=()=>{if(detail.open&&detail.childNodes.length===1)detail.append(el('pre',r.body,'source-body'));};section.append(detail);}else section.append(el('pre',r.body,'source-body'));return section;}));const failures=p.failures.map(f=>f.reference.kind+':'+f.reference.id+': '+f.error).join('\n');AC_I18N.bind(q('mcp-failures'),()=>failures);q('mcp-failures').hidden=!failures;q('mcp-apply').disabled=!p.records.length||p.failures.some(f=>f.reference.kind==='jira');previewing=false;q('mcp-loading').hidden=true;rememberNotes(p);if(!q('mcp-review').open)q('mcp-review').showModal();}
 function renderSuggestions(suggestions){AC_I18N.replace(q('source-suggestions'),...suggestions.map(ref=>{const b=el('button',()=>acMessage('mcp.use',{value1:ref}),'quiet small-button');b.type='button';b.onclick=()=>{const[k,id]=ref.split(':');q('source-kind').value=k;q('source-id').value=id;q('source-reference').showModal();};return b;}));q('source-suggestions').hidden=!suggestions.length;}
 function updateSourceSuggestions(goal){
  suggestionsStamp=goal;sourceCount=0;
  goalTags(goal,[]);
  if(!goal.trim()){renderSuggestions([]);return;}
  request({kind:'mcp_source_suggestions',goal}).then(result=>{
   if(suggestionsStamp!==goal)return;
   sourceCount=Array.isArray(result.references)?result.references.length:0;
   goalTags(goal,Array.isArray(result.reference_ranges)?result.reference_ranges:[]);
   renderSuggestions(Array.isArray(result.suggestions)?result.suggestions:[]);
  }).catch(()=>{if(suggestionsStamp===goal)renderSuggestions([]);});
 }
 function update(){
  const s=state();if(!s)return;const job=s.job,goal=draft()?.goal??s.config.goal;
  // An MCP lookup in progress is visible beside the source buttons and in the open windows.
  const looking=job?.kind==='mcp'&&job.state==='running';q('source-status').classList?.toggle('mcp-busy',looking);
  if(looking&&pendingDiscovery)AC_I18N.text(q('mcp-tools'),()=>acMessage('mcp.loading_tools'));
  if(job?.kind==='mcp'&&['failed','interrupted','stopped'].includes(job.state)&&job.id!==failedShown&&(previewing||pendingDiscovery)){
   failedShown=job.id;const error=String(job.error||'').replace(/^agentconductor:\s*/,'').trim();
   if(previewing){previewing=false;q('mcp-loading').hidden=true;AC_I18N.bind(q('mcp-failures'),()=>acMessage('mcp.lookup_failed',{value1:error||job.state}));q('mcp-failures').hidden=false;}
   else{pendingDiscovery=null;AC_I18N.text(q('mcp-tools'),()=>acMessage('mcp.lookup_failed',{value1:error||job.state}));}
  }
  if(goal!==suggestionsStamp)updateSourceSuggestions(goal);
  q('mcp-preview').disabled=job?.state==='running'||!sourceCount;
  AC_I18N.bind(q('source-status'),()=>looking?acMessage('mcp.lookup_running'):s.imports?acMessage('mcp.applied_sources',{value1:s.imports.selected.length}):sourceCount?acMessage('mcp.references_to_preview',{value1:sourceCount}):'');
  q('mcp-settings').disabled=job?.state==='running';
  if(job?.kind==='mcp'&&job.state==='completed'&&job.id!==shown&&s.mcp_result){
   shown=job.id;const result=s.mcp_result;
   if(result.tools){const server=result.server,config=state().config.mcp[server];if(config&&(!pendingDiscovery||pendingDiscovery.server===server&&pendingDiscovery.config===JSON.stringify(config)))catalogs.set(server,{tools:result.tools,config:JSON.stringify(config)});pendingDiscovery=null;servers();renderCatalog();}
   else showPreview(result);
  }
 }
 return {update,wantsResult:job=>job?.kind==='mcp'&&job.state==='completed'&&job.id!==shown};
}
