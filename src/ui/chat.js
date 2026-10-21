// DD-171 (GitHub #115): chat with the coordinator from the workspace screen; the answer arrives with the job snapshot.
function chatUi({q,request,state,notice}) {
 const turns=[];let waiting=null;
 const box=()=>q('start-chat');
 function setOpen(open){box().hidden=!open;q('start-chat-button').setAttribute('aria-expanded',String(open));if(open)q('start-chat-input').focus();}
 function turnNode(t){const d=document.createElement('div');d.className='chat-turn '+(t.error?'error':t.role);const h=document.createElement('strong');h.textContent=t.role==='operator'?acMessage('startup.chat_you'):acMessage('startup.chat_coordinator');const p=document.createElement('p');p.textContent=t.text;d.append(h,p);return d;}
 function draw(){const nodes=turns.map(turnNode);if(waiting){const d=document.createElement('div');d.className='chat-turn';const p=document.createElement('p');p.textContent=acMessage('startup.chat_working');d.append(p);nodes.push(d);}q('start-chat-log').replaceChildren(...nodes);q('start-chat-log').scrollTop=q('start-chat-log').scrollHeight;q('start-chat-send').disabled=!!waiting;}
 const sendable=()=>turns.filter(t=>!t.error).map(({role,text})=>({role,text}));
 async function send(){const text=q('start-chat-input').value.trim();if(!text||waiting)return;turns.push({role:'operator',text});q('start-chat-input').value='';waiting={};draw();try{const r=await request({kind:'chat',revision:state().revision,turns:sendable()});waiting={job:r.id};}catch(e){waiting=null;turns.pop();q('start-chat-input').value=text;notice(e.message,true);}draw();}
 function update(){const s=state();if(!waiting?.job||!s?.job||s.job.id!==waiting.job)return;const job=s.job;if(job.state==='completed'&&s.chat?.job===job.id){turns.push({role:'assistant',text:s.chat.answer});waiting=null;draw();}else if(['failed','interrupted','stopped'].includes(job.state)){turns.push({role:'assistant',error:true,text:acMessage('startup.chat_error',{value1:job.error||job.state})});waiting=null;draw();}}
 q('start-chat-button').onclick=()=>setOpen(box().hidden);
 q('start-chat-close').onclick=()=>setOpen(false);
 q('start-chat-form').onsubmit=e=>{e.preventDefault();send();};
 q('start-chat-input').addEventListener('keydown',e=>{if((e.metaKey||e.ctrlKey)&&e.key==='Enter'&&!e.isComposing){e.preventDefault();send();}else if(e.key==='Escape')setOpen(false);});
 return {update,turns:()=>turns.slice()};
}
