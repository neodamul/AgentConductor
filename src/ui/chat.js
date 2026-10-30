// DD-171 (GitHub #115) and DD-190 (#185): coordinator chat on the workspace screen as a persistent, terminal-style transcript; the server keeps the conversation and the answer arrives with the chat job's snapshot (GitHub #261: its own slot, next to a plan).
function chatUi({q,request,state,notice}) {
 let turns=[],waiting=null,loading=null,ready=false,recall=-1,draft='',unsure=null;
 const box=()=>q('chat');
 function setOpen(open){box().hidden=!open;q('chat-button').setAttribute('aria-expanded',String(open));if(open)q('chat-input').focus();}
 const node=(tag,cls,text)=>{const n=document.createElement(tag);if(cls)n.className=cls;if(text!==undefined)n.textContent=text;return n;};
 // Fenced code blocks keep their layout in <pre>; other text keeps its line breaks.
 function blocks(text){return text.split(/^```[^\n]*\n?/m).flatMap((part,i)=>part.trim()?[i%2?node('pre','chat-code',part.replace(/\n$/,'')):node('p','',part.trim())]:[]);}
 function turnNode(t){const d=node('div','chat-turn '+(t.error?'error':t.role));const h=node('strong','',t.role==='operator'?acMessage('startup.chat_you'):acMessage('startup.chat_coordinator'));d.append(h);if(t.role==='operator'){const p=node('p');p.append(node('span','chat-prompt','›'),t.text);d.append(p);}else d.append(...blocks(t.text));return d;}
 function draw(){const nodes=turns.map(turnNode);if(waiting){nodes.push(node('div','chat-turn working',acMessage('startup.chat_working')+' '+Math.round((Date.now()-waiting.at)/1000)+'s'));}q('chat-log').replaceChildren(...nodes);q('chat-log').scrollTop=q('chat-log').scrollHeight;q('chat-send').disabled=!!waiting;q('chat-new').disabled=!!waiting||!turns.length;}
 const sendable=()=>turns.filter(t=>t.role==='operator').map(t=>t.text);
 function load(){if(!loading)loading=request({kind:'chat_history'}).then(r=>{if(unsure)reconcile(r.turns);else if(!waiting)turns=r.turns.map(({role,text})=>({role,text}));ready=true;settle();draw();}).catch(()=>{loading=null;});return loading;}
 async function send(){const text=q('chat-input').value.trim();if(!text||waiting)return;await load();if(waiting)return;const before=sendable().length;turns.push({role:'operator',text});q('chat-input').value='';recall=-1;waiting={at:Date.now()};draw();try{const r=await request({kind:'chat',revision:state().chat_revision,text});waiting={job:r.id,at:waiting.at};}catch(e){await resync(text,e,before);}draw();}
 // A timed-out send may have started a job; failed history reads keep the outcome pending until it can be reconciled.
 async function resync(text,error,before){unsure={text,error,before};loading=null;ready=false;await load();}
 function reconcile(stored){const {text:sent,error,before}=unsure;unsure=null;waiting=null;turns=stored.map(({role,text})=>({role,text}));if(stored.filter(t=>t.role==='operator').length>before)return;const typed=q('chat-input').value;q('chat-input').value=sent+(typed?'\n'+typed:'');turns.push({role:'assistant',error:true,text:error.message});}
 // Follows the job: shows the answer or the failure once, and picks up a chat that was running when the screen reopened.
 function settle(){const s=state();if(!s||!ready)return;const job=s.chat_job;if(waiting&&!waiting.job)return;if(waiting){if(job?.id!==waiting.job)return;if(job.state==='completed'&&s.chat?.job===job.id){turns.push({role:'assistant',text:s.chat.answer});waiting=null;}else if(['failed','interrupted','stopped'].includes(job.state)){turns.push({role:'assistant',error:true,text:acMessage('startup.chat_error',{value1:job.error||job.state})});waiting=null;}draw();return;}
  if(turns.at(-1)?.role!=='operator'||job?.kind!=='chat')return;if(job.state==='running'){waiting={job:job.id,at:Date.now()};draw();}else if(job.state==='completed'&&s.chat?.job===job.id){turns.push({role:'assistant',text:s.chat.answer});draw();}}
 function update(){if(!state())return;if(!ready){load();return;}settle();if(waiting)draw();}
 // A different workspace has its own conversation.
 function reset(){turns=[];waiting=null;unsure=null;loading=null;ready=false;recall=-1;draw();}
 function step(dir){const h=sendable();if(!h.length)return;let i=(recall<0?h.length:recall)+dir;if(i<0)return;if(recall<0)draft=q('chat-input').value;if(i>=h.length){recall=-1;q('chat-input').value=draft;return;}recall=i;q('chat-input').value=h[i];}
 q('chat-button').onclick=()=>setOpen(box().hidden);
 q('chat-close').onclick=()=>setOpen(false);
 q('chat-new').onclick=async()=>{if(waiting)return;try{await request({kind:'chat_reset'});turns=[];recall=-1;draw();}catch(e){notice(e.message,true);}};
 q('chat-form').onsubmit=e=>{e.preventDefault();send();};
 // Enter sends and Shift+Enter breaks the line, like a terminal prompt; Up/Down recall earlier questions.
 q('chat-input').addEventListener('keydown',e=>{if(e.isComposing)return;if(e.key==='Enter'&&!e.shiftKey&&!e.altKey){e.preventDefault();send();}else if(e.key==='Escape')setOpen(false);else if(e.key==='ArrowUp'&&(recall>=0||!q('chat-input').value)){e.preventDefault();step(-1);}else if(e.key==='ArrowDown'&&recall>=0){e.preventDefault();step(1);}});
 return {update,reset,turns:()=>turns.slice()};
}
