// #427: one full briefing renderer for the Workspace and the Room; no polling or LLM call.
window.acBriefing=(()=>{
 const $=id=>document.getElementById(id);
 const node=(tag,text,cls)=>{const n=document.createElement(tag);if(text!==undefined)AC_I18N.label(n,text);if(cls)n.className=cls;return n;};
 const pretty=value=>{const key='state.'+String(value||'unknown');return Object.hasOwn(AC_MESSAGES.en,key)?acMessage(key):String(value||acMessage('room.unknown')).replaceAll('_',' ');};
 document.body.append($('briefing-dialog'));
async function open({request,notice}){
 try{const b=await request({kind:'briefing'});AC_I18N.bind($('briefing-time'),()=>(acMessage("room.captured_read_only", {value1:pretty(b.state),value2:new Date(b.observed_at_ms).toLocaleString()})));
 AC_I18N.replace($('briefing-sessions'),...b.sessions.map(s=>{const card=node('article',undefined,'briefing-session');card.append(node('h3',`${s.agent} · ${s.task} · ${pretty(s.state)}`),node('p',s.work),node('p',()=>(acMessage("room.session_model_attempt", {value1:s.session_id||acMessage("room.not_established"),value2:s.model||acMessage("startup.cli_default"),value3:s.attempt})),'small muted'),node('p',()=>(acMessage("room.process_observed", {value1:pretty(s.process.state),value2:new Date(s.process.observed_at_ms).toLocaleTimeString()})),'small muted'));
 if(s.last_activity)card.append(node('p',()=>(acMessage("room.last_activity", {value1:pretty(s.last_activity.kind),value2:pretty(s.last_activity.freshness),value3:s.last_activity.observed_at_ms?' · '+new Date(s.last_activity.observed_at_ms).toLocaleTimeString():''})),'small muted'),node('p',s.last_activity.text,'briefing-text'));
 const memory=s.memory_report;card.append(node('p',()=>(memory.summary?acMessage("room.memory_report", {value1:memory.source,value2:memory.freshness,value3:memory.summary}):memory.note),'small muted'));
 if(s.restriction)card.append(node('p',()=>(s.restriction.available_at?acMessage("room.available", {value1:new Date(s.restriction.available_at*1000).toLocaleString()}):acMessage("room.availability_reset_unknown")),'warning'));
 if(s.attention)card.append(node('p',s.attention,'warning'));card.append(node('p',()=>(acMessage("room.next")+s.next_action)));return card;}));
 AC_I18N.bind($('briefing-approvals'),()=>(b.approvals.length?acMessage("room.review_requests")+b.approvals.map(a=>`${a.kind} · ${a.task||acMessage("room.integration_candidate")} · ${a.status}`).join('; '):acMessage("room.no_recorded_approval_requests")));
 $('briefing-dialog').showModal();}catch(e){notice(e.message,true);}
}
 return {open};
})();
