const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
class Node {
 constructor(tag='DIV'){this.nodeType=1;this.tagName=tag.toUpperCase();this.childNodes=[];this.attrs={};this.dataset={};this.style={};this.value='';this.hidden=false;this.open=false;this.checked=false;this.required=false;this._text='';this.handlers={};this.classList={add(){},toggle(){}};}
 getAttribute(k){return this.attrs[k]??null;}setAttribute(k,v){this.attrs[k]=String(v);}removeAttribute(k){delete this.attrs[k];}
 get children(){return this.childNodes;}
 get textContent(){return this.childNodes.length?this.childNodes.map(n=>n.textContent).join(''):this._text;}set textContent(v){this.childNodes=[];this._text=String(v);}
 append(...nodes){for(const n of nodes){n.parentNode=this;this.childNodes.push(n);}}replaceChildren(...nodes){this.childNodes=[];this._text='';this.append(...nodes);if(this.tagName==='SELECT')this.value=nodes[0]?.value||'';}
 showModal(){this.open=true;}close(){this.open=false;}click(){return this.onclick?.();}focus(){}select(){}remove(){this.parentNode?.childNodes.splice(this.parentNode.childNodes.indexOf(this),1);}addEventListener(k,fn){this.handlers[k]=fn;}
 querySelectorAll(selector){const match=n=>selector==='.mcp-secret'?n.className==='mcp-secret':selector==='[data-argument]'?n.dataset?.argument!==undefined:selector==='[data-id-field]'?n.dataset?.idField!==undefined:selector==='input:checked'?n.checked:false;return this.childNodes.flatMap(n=>[...(match(n)?[n]:[]),...(n.querySelectorAll?.(selector)||[])]);}
 querySelector(selector){return this.querySelectorAll(selector)[0]||null;}
}
const body=new Node('BODY'),document={body,documentElement:{lang:'en'},dispatchEvent(){},createTextNode:text=>({nodeType:3,childNodes:[],textContent:text}),createElement:tag=>new Node(tag)};
const context={document,Event:class{},AC_MESSAGES:{en:JSON.parse(fs.readFileSync('src/ui/messages/en.json','utf8')),ko:JSON.parse(fs.readFileSync('src/ui/messages/ko.json','utf8'))},requestAnimationFrame:fn=>fn()};
const i18n=fs.readFileSync('src/ui/i18n.js','utf8'),elapsed=i18n.split('\n').find(l=>l.startsWith('const AC_ELAPSED='));assert.ok(elapsed,'AC_ELAPSED exists');
const runtime=vm.runInNewContext(i18n.split('// DD-111:')[0]+elapsed+';({AC_I18N,acMessage,AC_ELAPSED})',context);Object.assign(context,runtime);
const {mcpUi}=vm.runInNewContext(fs.readFileSync('src/ui/mcp.js','utf8')+';({mcpUi})',context);
const readTool={name:'getJiraIssue',annotations:{readOnlyHint:true},inputSchema:{type:'object',properties:{issueIdOrKey:{type:'string'},cloudId:{type:'string'},includeArchived:{type:'boolean'}},required:['issueIdOrKey','cloudId','includeArchived']}};
const writeTool={name:'updateIssue',annotations:{readOnlyHint:false},inputSchema:{type:'object',properties:{key:{type:'string'}}}};
const fields=new Map();function q(id){if(!fields.has(id)){const tag=['mcp-kind','mcp-jira','mcp-confluence'].includes(id)?'SELECT':'DIV';fields.set(id,new Node(tag));body.append(fields.get(id));}return fields.get(id);}
function el(tag,text,cls){const n=new Node(tag);if(text!==undefined)runtime.AC_I18N.label(n,text);n.className=cls;return n;}
let state={config:{mcp:{atlassian:{type:'remote',url:'https://mcp.example',enabled:true}},goal_sources:{},goal:'jira:EMM-1',coordinator:'planner',profiles:[{id:'planner'}]},job:null},draft={...state.config};
const packets=[],notices=[];
async function action(packet){packets.push(packet);if(packet.kind==='mcp_save'){const s=JSON.parse(packet.settings);state.config={...state.config,...s};draft={...draft,...s};state.job=null;}if(packet.kind==='mcp_discover')state.job={kind:'mcp',state:'running',id:'discover'};return {saved:true};}
const ui=mcpUi({q,el,request:async()=>({references:[],reference_ranges:[],suggestions:[]}),action,save:async()=>true,mark(){},capture(){},poll(){},state:()=>state,draft:()=>draft,notice:text=>notices.push(typeof text==='function'?text():text)});
(async()=>{
 q('mcp-settings').onclick();assert.equal(q('mcp-servers').childNodes.length,1,'registered server is listed');
 q('mcp-add').onclick();q('mcp-name').value='local-atlas';q('mcp-kind').value='local';q('mcp-kind').onchange();q('mcp-executable').value='atlas-mcp';q('mcp-arguments').value='--stdio\n--safe';
 await q('mcp-form').onsubmit({preventDefault(){}});assert.deepEqual(state.config.mcp['local-atlas'].command,['atlas-mcp','--stdio','--safe'],'arguments stay separate');assert.equal(packets.at(-1).kind,'mcp_discover','saving discovers immediately');
 state.job={kind:'mcp',state:'completed',id:'discover'};state.mcp_result={server:'local-atlas',tools:[readTool,writeTool]};ui.update();assert.equal(q('mcp-jira').childNodes.length,2,'only one read-only discovered tool can be bound');assert.match(q('mcp-jira').childNodes[1].textContent,/getJiraIssue/);
 q('mcp-jira').value=JSON.stringify({server:'local-atlas',tool:'getJiraIssue'});q('mcp-jira').onchange();assert.equal(q('mcp-jira-arguments').querySelector('[data-id-field]').value,'issueIdOrKey');const args=q('mcp-jira-arguments').querySelectorAll('[data-argument]');args[0].value='tenant-a';args[1].checked=true;
 await q('mcp-bindings-save').onclick();const binding=state.config.goal_sources.jira;assert.deepEqual(binding,{server:'local-atlas',tool:'getJiraIssue',arguments:{issueIdOrKey:'{id}',cloudId:'tenant-a',includeArchived:true},read_only:true});
 assert(q('mcp-json').value.includes('local-atlas'),'advanced JSON export reflects saved settings');
 const list=q('mcp-servers').childNodes;assert.equal(list.length,2);console.log('MCP server registration, immediate discovery, read-only binding, required arguments, and JSON export passed');
})().catch(e=>{console.error(e);process.exitCode=1;});
