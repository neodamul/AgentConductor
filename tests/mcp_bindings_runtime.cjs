const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
class Node {
 constructor(tag='DIV'){this.nodeType=1;this.tagName=tag;this.childNodes=[];this.attrs={};this.value='';this.hidden=false;this.open=false;this.checked=false;this._text='';}
 getAttribute(k){return this.attrs[k]??null;}setAttribute(k,v){this.attrs[k]=String(v);}removeAttribute(k){delete this.attrs[k];}
 get textContent(){return this.childNodes.length?this.childNodes.map(n=>n.textContent).join(''):this._text;}set textContent(v){this.childNodes=[];this._text=String(v);}
 append(...nodes){this.childNodes.push(...nodes);}replaceChildren(...nodes){this.childNodes=nodes;this._text='';if(this.tagName==='SELECT')this.value=nodes[0]?.value||'';}
 showModal(){this.open=true;}close(){this.open=false;}click(){return this.onclick?.();}scrollIntoView(){}querySelectorAll(){return [];}
}
const body=new Node('BODY'),document={body,documentElement:{lang:'en'},dispatchEvent(){},createTextNode:text=>({nodeType:3,childNodes:[],textContent:text})};
const context={document,Event:class{},AC_MESSAGES:{en:JSON.parse(fs.readFileSync('src/ui/messages/en.json','utf8')),ko:JSON.parse(fs.readFileSync('src/ui/messages/ko.json','utf8'))},requestAnimationFrame:fn=>fn()};
const runtime=vm.runInNewContext(fs.readFileSync('src/ui/i18n.js','utf8').split('// DD-111:')[0]+';({AC_I18N,acMessage})',context);Object.assign(context,runtime);
const {mcpUi}=vm.runInNewContext(fs.readFileSync('src/ui/mcp.js','utf8')+';({mcpUi})',context);
const jira={name:'getJiraIssue',annotations:{readOnlyHint:true},inputSchema:{type:'object',properties:{issueIdOrKey:{type:'string'},cloudId:{type:'string'}},required:['issueIdOrKey','cloudId']}};
const fields=new Map();function q(id){if(!fields.has(id)){const n=new Node(id==='mcp-server'?'SELECT':'DIV');fields.set(id,n);body.append(n);}return fields.get(id);}
function el(tag,text,cls){const n=new Node(tag.toUpperCase());if(text!==undefined)runtime.AC_I18N.label(n,text);n.className=cls;return n;}
let state={config:{mcp:{atlassian:{type:'local',command:['company-mcp']},other:{type:'local',command:['other-mcp']}},goal_sources:{},goal:'jira:EMM-1 confluence:123',coordinator:'planner',profiles:[{id:'planner'}]},job:null};
const draft={...state.config},packets=[],notices=[],uiRequests=[];
async function action(packet){packets.push(packet);if(packet.kind==='mcp_save'){const s=JSON.parse(packet.settings);state.config={...state.config,...s};state.job=null;state.mcp_result=null;}return {saved:true};}
const ui=mcpUi({q,el,request:async packet=>{uiRequests.push(packet);return packet.kind==='mcp_source_suggestions'?{references:['jira:EMM-1'],suggestions:['jira:EMM-1']}:{};},action,save:async()=>true,mark(){},capture(){},poll(){},state:()=>state,draft:()=>draft,notice:text=>notices.push(typeof text==='function'?text():text)});
(async()=>{
 await q('mcp-preview').onclick();assert.equal(packets.at(-1).kind,'mcp_preview','coordinator uses registered servers without manual bindings');
 draft.coordinator='missing';const before=packets.length;await q('mcp-preview').onclick();assert.equal(packets.length,before);assert.match(notices.at(-1),/Register an agent CLI/);draft.coordinator='planner';
 q('mcp-settings').onclick();q('mcp-server').value='other';q('mcp-server').onchange();assert.equal(JSON.parse(q('mcp-server-json').value).command[0],'other-mcp');
 await q('mcp-discover').onclick();assert.equal(packets.at(-1).server,'other');
 q('mcp-server').value='atlassian';q('mcp-server').onchange();state.job={kind:'mcp',state:'completed',id:'catalog-other'};state.mcp_result={server:'other',tools:[jira]};ui.update();await Promise.resolve();assert(uiRequests.some(p=>p.kind==='mcp_source_suggestions'&&p.goal===state.config.goal),'source suggestions are parsed by native Rust');assert.equal(q('source-suggestions').childNodes.length,1);assert.equal(q('mcp-server').value,'atlassian','late discovery must not switch selected server');assert(!q('mcp-tools').textContent.includes('getJiraIssue'));
 q('mcp-server').value='other';q('mcp-server').onchange();assert.match(q('mcp-tools').textContent,/getJiraIssue/);assert.match(q('mcp-tools').textContent,/other/);
 q('mcp-server-json').value=JSON.stringify({type:'local',command:['changed']});await q('mcp-server-save').onclick();assert.equal(q('mcp-server').value,'other');assert.equal(state.config.mcp.atlassian.command[0],'company-mcp');assert.equal(state.config.mcp.other.command[0],'changed');assert(!q('mcp-tools').textContent.includes('getJiraIssue'),'changed server invalidates its catalog');
 assert.equal(fields.has('source-binding-save'),false,'ordinary UI never asks user to select document tools');console.log('Coordinator selection, missing coordinator, per-server configuration/catalog, late result and stale cache checks passed');
})().catch(e=>{console.error(e);process.exitCode=1;});
