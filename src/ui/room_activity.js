// R-120 / DD-129: consecutive thinking updates read as one line; events carry no time, so the line names their range.
const thinkingUpdate=e=>{const r=e?.payload||{};return r.type==='thinking_tokens'||(r.type==='system'&&r.subtype==='thinking_tokens');};
function collapseThinking(events){const out=[];for(const e of events){const last=out.at(-1);if(!thinkingUpdate(e))out.push({event:e});else if(last?.thinking){last.thinking.count++;last.thinking.last=e.seq;}else out.push({thinking:{count:1,first:e.seq,last:e.seq}});}return out;}
function interruptionText(list){if(!list?.length)return '';const last=list.at(-1);return acMessage("room.interrupted_recovered",{value1:list.length,value2:new Date(last.at).toLocaleTimeString(),value3:pretty(last.status)});}
