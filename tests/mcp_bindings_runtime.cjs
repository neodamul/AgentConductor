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
const {mcpUi,mcpSourceArguments,mcpMissingBindings,mcpReadTool}=vm.runInNewContext(fs.readFileSync('src/ui/mcp.js','utf8')+';({mcpUi,mcpSourceArguments,mcpMissingBindings,mcpReadTool})',context);
const json=value=>JSON.parse(JSON.stringify(value));
const jira={name:'getJiraIssue',annotations:{readOnlyHint:true},inputSchema:{type:'object',properties:{issueIdOrKey:{type:'string'},cloudId:{type:'string'}},required:['issueIdOrKey','cloudId']}};
const confluence={name:'getConfluencePage',annotations:{readOnlyHint:true},inputSchema:{type:'object',properties:{id:{type:'string'},cloudId:{type:'string'}},required:['id','cloudId']}};
assert.deepEqual(json(mcpSourceArguments('jira',jira)),{issueIdOrKey:'{id}',cloudId:''});
assert.deepEqual(json(mcpSourceArguments('confluence',confluence)),{id:'{id}',cloudId:''});
assert.deepEqual(json(mcpSourceArguments('jira',{inputSchema:{properties:{issue_key:{type:'string'}},required:['issue_key']}})),{issue_key:'{id}'});
assert.deepEqual(json(mcpMissingBindings('jira:EMM-1 confluence:123 jira:EMM-2',{})),['jira','confluence']);
assert.deepEqual(json(mcpMissingBindings('jira:EMM-1',{jira:{}})),[]);
assert.equal(mcpReadTool({name:'write',annotations:{readOnlyHint:false}}),false);
assert.equal(mcpReadTool({name:'remove',annotations:{destructiveHint:true,readOnlyHint:true}}),false);
const fields=new Map();function q(id){if(!fields.has(id)){const n=new Node(id==='mcp-server'?'SELECT':'DIV');fields.set(id,n);body.append(n);}return fields.get(id);}
function el(tag,text,cls){const n=new Node(tag.toUpperCase());if(text!==undefined)runtime.AC_I18N.label(n,text);n.className=cls;return n;}
let state={config:{mcp:{atlassian:{type:'local',command:['company-mcp']}},goal_sources:{},goal:'jira:EMM-1 confluence:123'},job:null};
const draft={...state.config},packets=[],notices=[];
async function action(packet){packets.push(packet);if(packet.kind==='mcp_save'){const s=JSON.parse(packet.settings);state.config={...state.config,...s};state.job=null;state.mcp_result=null;}return {saved:true};}
const ui=mcpUi({q,el,request:async()=>({}),action,save:async()=>true,mark(){},capture(){},poll(){},state:()=>state,draft:()=>draft,notice:text=>notices.push(typeof text==='function'?text():text)});
(async()=>{
 await q('mcp-preview').onclick();assert.equal(packets.length,0);assert.equal(q('mcp-dialog').open,true);assert.match(notices.at(-1),/not connected/);
 state.job={kind:'mcp',state:'completed',id:'catalog-1'};state.mcp_result={server:'atlassian',tools:[jira,confluence,{name:'deleteIssue',annotations:{destructiveHint:true}}]};ui.update();
 const rows=q('mcp-tool-bindings').childNodes;assert.equal(rows.length,3);assert.equal(rows[2].childNodes[1].disabled,true);
 rows[0].childNodes[1].onclick();assert.equal(q('source-binding-editor').hidden,false);assert.deepEqual(JSON.parse(q('source-binding-arguments').value),{issueIdOrKey:'{id}',cloudId:''});
 q('source-binding-arguments').value=JSON.stringify({issueIdOrKey:'{id}',cloudId:'tenant-1'});await q('source-binding-save').onclick();assert.equal(state.config.goal_sources.jira.tool,'getJiraIssue');assert.equal(draft.goal_sources.jira.arguments.cloudId,'tenant-1');
 rows[1].childNodes[2].onclick();q('source-binding-arguments').value=JSON.stringify({id:'{id}',cloudId:'tenant-1'});await q('source-binding-save').onclick();assert.equal(state.config.goal_sources.confluence.tool,'getConfluencePage');assert.equal(state.config.goal_sources.jira.tool,'getJiraIssue');
 await q('mcp-preview').onclick();assert.equal(packets.at(-1).kind,'mcp_preview');
 rows[0].childNodes[1].onclick();q('source-binding-read-only').checked=false;const count=packets.length;await q('source-binding-save').onclick();assert.equal(packets.length,count);assert.match(notices.at(-1),/Confirm/);
 q('source-binding-read-only').checked=true;state.config.mcp.atlassian={type:'local',command:['different-mcp']};await q('source-binding-save').onclick();assert.equal(packets.length,count);assert.match(notices.at(-1),/configuration changed/);
 q('mcp-server').onchange();assert.equal(q('mcp-tool-bindings').childNodes.length,0);q('source-binding-status').childNodes[0].childNodes[1].onclick();assert.equal(packets.at(-1).kind,'mcp_discover');
 console.log('Server registration, discovery, binding both sources, preview routing, read-only and stale-catalog checks passed');
})().catch(e=>{console.error(e);process.exitCode=1;});
