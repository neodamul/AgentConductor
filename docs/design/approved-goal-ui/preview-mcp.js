// MCP servers: a list of saved servers, a short add/edit form, automatic tool discovery; document tools are selected by the coordinator.
// Fixture servers only; production stores the same fields and discovers each server's real read-only tools.
const mcpServers=[
 {name:'atlassian',kind:'remote',url:'https://mcp.example.internal/atlassian',command:'',secrets:[['Authorization','Bearer {env:ATLASSIAN_TOKEN}']],state:'ok',tools:['jira_get_issue','jira_search','confluence_get_page','confluence_search']},
 {name:'wiki-local',kind:'local',url:'',command:'npx -y @company/wiki-mcp',secrets:[['WIKI_TOKEN','{env:WIKI_TOKEN}']],state:'error',error:'명령을 찾을 수 없습니다: npx',tools:[]}
];
const mcpBindings={jira:'atlassian · jira_get_issue',confluence:'atlassian · confluence_get_page'};
const mcpDialog=document.getElementById('mcp-dialog'),mcpField=name=>document.getElementById('mcp-'+name);
let mcpEditing=null;
function mcpKindLabel(server){return server.kind==='remote'?'원격 URL':'로컬 명령';}
function mcpStatus(server){return server.state==='checking'?'도구 확인 중…':server.state==='ok'?'도구 '+server.tools.length+'개 확인됨':'연결 실패 · '+server.error;}
function renderMcpServers(){
 const list=mcpField('servers');list.replaceChildren();
 if(!mcpServers.length){const empty=document.createElement('p');empty.className='panel-note';empty.textContent='등록된 서버가 없습니다. + 서버 추가로 시작하세요.';list.append(empty);}
 mcpServers.forEach((server,index)=>{
  const row=document.createElement('div');row.className='mcp-server';row.setAttribute('role','listitem');
  const info=document.createElement('div'),name=document.createElement('strong'),meta=document.createElement('small');
  name.textContent=server.name;meta.textContent=mcpKindLabel(server)+' · '+(server.kind==='remote'?server.url:server.command);meta.title=meta.textContent;
  const status=document.createElement('span');status.className='mcp-status '+server.state;status.textContent=mcpStatus(server);status.setAttribute('role','status');
  info.append(name,meta,status);
  const actions=document.createElement('div');actions.className='mcp-server-actions';
  const button=(label,run,title)=>{const b=document.createElement('button');b.type='button';b.className='btn q';b.textContent=label;b.title=title;b.onclick=run;b.disabled=server.state==='checking';actions.append(b);return b;};
  button('편집',()=>openMcpEditor(index),server.name+' 편집');
  button('다시 확인',()=>discoverMcp(server),server.name+' 도구 다시 확인');
  button('삭제',()=>{mcpServers.splice(index,1);renderMcp();},server.name+' 삭제');
  row.append(info,actions);list.append(row);
 });
}
function renderMcp(){renderMcpServers();}
function discoverMcp(server){
 server.state='checking';renderMcp();
 setTimeout(()=>{
  const reachable=server.kind==='remote'?/^https:\/\//.test(server.url):!/^npx\b/.test(server.command);
  Object.assign(server,reachable?{state:'ok',error:'',tools:server.tools.length?server.tools:['jira_get_issue','confluence_get_page']}:{state:'error',error:server.kind==='remote'?'https 주소가 아닙니다':'명령을 찾을 수 없습니다: '+server.command.split(' ')[0],tools:[]});
  renderMcp();
 },mcpDiscoverDelay);
}
let mcpDiscoverDelay=700;
function showMcpView(edit){
 mcpField('list-view').hidden=edit;mcpField('edit-view').hidden=!edit;mcpField('list-actions').hidden=edit;mcpField('edit-actions').hidden=!edit;
 mcpField('title').textContent=edit?(mcpEditing===null?'MCP 서버 추가':mcpServers[mcpEditing].name+' · 편집'):'MCP 서버';
}
function addMcpSecret(name='',value=''){
 const row=document.createElement('div');row.className='mcp-secret';
 const key=document.createElement('input'),val=document.createElement('input'),remove=document.createElement('button');
 key.value=name;key.setAttribute('aria-label','이름');key.placeholder=mcpField('kind').value==='remote'?'Authorization':'ATLASSIAN_TOKEN';
 val.value=value;val.setAttribute('aria-label','값');val.placeholder='{env:ATLASSIAN_TOKEN}';
 remove.type='button';remove.className='btn q';remove.textContent='×';remove.setAttribute('aria-label','이 값 삭제');remove.onclick=()=>row.remove();
 row.append(key,val,remove);mcpField('secrets').append(row);
}
function syncMcpKind(){
 const remote=mcpField('kind').value==='remote';
 mcpField('url-label').hidden=!remote;mcpField('command-label').hidden=remote;
 mcpField('url').required=remote;mcpField('command').required=!remote;
 mcpField('secret-title').textContent=remote?'헤더':'환경 변수';
}
function openMcpEditor(index){
 mcpEditing=index;const server=index===null?{name:'',kind:'remote',url:'',command:'',secrets:[]}:mcpServers[index];
 mcpField('name').value=server.name;mcpField('kind').value=server.kind;mcpField('url').value=server.url;mcpField('command').value=server.command;
 mcpField('secrets').replaceChildren();syncMcpKind();server.secrets.forEach(([name,value])=>addMcpSecret(name,value));if(!server.secrets.length)addMcpSecret();
 mcpField('edit-error').textContent='';showMcpView(true);mcpField('name').focus();
}
function openMcp(){mcpEditing=null;showMcpView(false);renderMcp();if(!mcpDialog.open)mcpDialog.showModal();}
mcpField('kind').onchange=syncMcpKind;
mcpField('secret-add').onclick=()=>addMcpSecret();
mcpField('add').onclick=()=>openMcpEditor(null);
mcpField('edit-cancel').onclick=()=>{mcpEditing=null;showMcpView(false);};
mcpField('close').onclick=()=>mcpDialog.close();
mcpField('open').onclick=()=>openMcp();
mcpField('form').onsubmit=event=>{
 event.preventDefault();
 const name=mcpField('name').value.trim(),kind=mcpField('kind').value;
 const taken=mcpServers.some((server,index)=>server.name===name&&index!==mcpEditing);
 if(!mcpField('form').reportValidity())return;
 if(taken){mcpField('edit-error').textContent='같은 이름의 서버가 이미 있습니다.';return;}
 const secrets=[...mcpField('secrets').children].map(row=>[...row.querySelectorAll('input')].map(input=>input.value.trim())).filter(([key])=>key);
 const server=Object.assign(mcpEditing===null?{tools:[]}:mcpServers[mcpEditing],{name,kind,url:mcpField('url').value.trim(),command:mcpField('command').value.trim(),secrets});
 if(mcpEditing===null)mcpServers.push(server);
 mcpEditing=null;showMcpView(false);discoverMcp(server);
};
mcpField('export').onclick=()=>{
 const mcp=Object.fromEntries(mcpServers.map(server=>[server.name,server.kind==='remote'?{type:'remote',url:server.url,headers:Object.fromEntries(server.secrets)}:{type:'local',command:server.command.split(' '),environment:Object.fromEntries(server.secrets)}]));
 mcpField('json').value=JSON.stringify({mcp,goal_sources:{jira:mcpBindings.jira,confluence:mcpBindings.confluence}},null,2);
};
mcpField('import').onclick=()=>{
 try{const parsed=JSON.parse(mcpField('json').value);Object.entries(parsed.mcp||{}).forEach(([name,value])=>{if(mcpServers.some(server=>server.name===name))return;const server={name,kind:value.type==='local'?'local':'remote',url:value.url||'',command:(value.command||[]).join(' '),secrets:Object.entries(value.headers||value.environment||{}),tools:[]};mcpServers.push(server);discoverMcp(server);});renderMcp();}
 catch{mcpField('json').setCustomValidity('JSON을 읽을 수 없습니다.');mcpField('json').reportValidity();mcpField('json').setCustomValidity('');}
};
