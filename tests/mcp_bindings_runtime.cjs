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
const fields=new Map();function q(id){assert(!['mcp-jira','mcp-confluence','mcp-jira-arguments','mcp-confluence-arguments','mcp-bindings-save','mcp-tools'].includes(id),'obsolete normal MCP control accessed: '+id);if(!fields.has(id)){const tag=['mcp-kind','mcp-jira','mcp-confluence'].includes(id)?'SELECT':'DIV';fields.set(id,new Node(tag));body.append(fields.get(id));}return fields.get(id);}
function el(tag,text,cls){const n=new Node(tag);if(text!==undefined)runtime.AC_I18N.label(n,text);n.className=cls;return n;}
let state={config:{mcp:{atlassian:{type:'remote',url:'https://mcp.example',enabled:true}},goal_sources:{},goal:'jira:EMM-1',coordinator:'planner',profiles:[{id:'planner'}]},job:null},draft={...state.config};
const packets=[],notices=[];
async function action(packet){packets.push(packet);if(packet.kind==='mcp_save'){const s=JSON.parse(packet.settings);state.config={...state.config,...s};draft={...draft,...s};state.job=null;}if(packet.kind==='mcp_discover')state.job={kind:'mcp',state:'running',id:'discover'};return {saved:true};}
const ui=mcpUi({q,el,request:async()=>({references:[],reference_ranges:[],suggestions:[]}),action,save:async()=>true,mark(){},capture(){},poll(){},state:()=>state,draft:()=>draft,notice:text=>notices.push(typeof text==='function'?text():text)});
(async()=>{
 q('mcp-settings').onclick();assert.equal(q('mcp-servers').childNodes.length,1,'registered server is listed');
 q('mcp-add').onclick();q('mcp-name').value='local-atlas';q('mcp-kind').value='local';q('mcp-kind').onchange();q('mcp-executable').value='atlas-mcp';q('mcp-arguments').value='--stdio\n--safe';
 await q('mcp-form').onsubmit({preventDefault(){}});assert.deepEqual(state.config.mcp['local-atlas'].command,['atlas-mcp','--stdio','--safe'],'arguments stay separate');assert.equal(packets.at(-1).kind,'mcp_discover','saving discovers immediately');
 state.job={kind:'mcp',state:'completed',id:'discover'};state.mcp_result={server:'local-atlas',tools:[readTool,writeTool]};ui.update();assert(q('mcp-servers').textContent.includes('2'),'discovery count stays visible');assert(!q('mcp-servers').textContent.includes('getJiraIssue'),'tool names are not duplicated as a text list');
 const legacy={server:'atlassian',tool:'getJiraIssue',arguments:{issueIdOrKey:'{id}',cloudId:'tenant-a'},read_only:true};state.config.goal_sources={jira:legacy};draft.goal_sources=state.config.goal_sources;
 q('mcp-export').onclick();assert.deepEqual(JSON.parse(q('mcp-json').value).goal_sources.jira,legacy,'internal bindings remain in exported configuration');await q('mcp-import').onclick();assert.deepEqual(state.config.goal_sources.jira,legacy,'configuration round trip keeps the internal binding');
 const row=q('mcp-servers').children.find(r=>r.children[0].children[0].textContent==='local-atlas');await row.children[1].children[1].click();state.job={kind:'mcp',state:'failed',id:'failed-discovery',error:'fixture connection refused'};ui.update();assert(q('mcp-servers').textContent.includes('fixture connection refused'),'server discovery failures remain actionable');
 await q('mcp-preview').onclick();assert.equal(packets.at(-1).kind,'mcp_preview','preview uses coordinator-selected routing without a binding form');
 assert(q('mcp-json').value.includes('local-atlas'),'advanced JSON export reflects saved settings');
 const list=q('mcp-servers').childNodes;assert.equal(list.length,2);console.log('MCP registration, discovery/status/errors, automatic preview and internal configuration round trip passed');
})().catch(e=>{console.error(e);process.exitCode=1;});
