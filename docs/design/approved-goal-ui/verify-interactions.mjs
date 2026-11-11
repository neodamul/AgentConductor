// Local design-reference checks using Chrome DevTools and Node's built-in WebSocket.
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdtemp,readFile,rm,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
const root=dirname(fileURLToPath(import.meta.url));
const folder=await mkdtemp(join(tmpdir(),'ac-ui-interactions-'));
const chrome=spawn('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',[
 '--headless=new','--disable-gpu','--no-first-run','--no-default-browser-check',
 '--remote-debugging-port=0','--user-data-dir='+folder,'about:blank'
],{stdio:'ignore'});
const pending=new Map(),listeners=new Map();let socket,nextId=0;
function call(method,params={}){
 const id=++nextId;
 return new Promise((resolve,reject)=>{
  const timer=setTimeout(()=>{pending.delete(id);reject(new Error(method+' timed out'));},15000);
  pending.set(id,{resolve:value=>{clearTimeout(timer);resolve(value);},reject:error=>{clearTimeout(timer);reject(error);}});
  socket.send(JSON.stringify({id,method,params}));
 });
}
function once(method){return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{listeners.delete(method);reject(new Error(method+' event timed out'));},15000);listeners.set(method,value=>{clearTimeout(timer);resolve(value);});});}
try{
 let port;
 for(let i=0;i<100;i++){
  try{port=(await readFile(join(folder,'DevToolsActivePort'),'utf8')).split('\n')[0];break;}catch{}
  await new Promise(resolve=>setTimeout(resolve,100));
 }
 if(!port)throw new Error('Chrome did not start');
 const targets=await (await fetch('http://127.0.0.1:'+port+'/json/list')).json();
 socket=new WebSocket(targets.find(target=>target.type==='page').webSocketDebuggerUrl);
 await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
 socket.addEventListener('message',event=>{
  const message=JSON.parse(event.data);
  if(message.id){const item=pending.get(message.id);pending.delete(message.id);if(item){if(message.error)item.reject(new Error(JSON.stringify(message.error)));else item.resolve(message.result);}}
  else if(listeners.has(message.method)){const resolve=listeners.get(message.method);listeners.delete(message.method);resolve(message.params);}
 });
 await call('Page.enable');
 const script=await readFile(join(root,'interaction-checks.js'),'utf8');
 const reports=[];
 for(const theme of ['light','dark'])for(const width of [840,1280]){
  await call('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
  const loaded=once('Page.loadEventFired');
  await call('Page.navigate',{url:pathToFileURL(join(root,'preview.html')).href+'?theme='+theme+'&run='+width});await loaded;
  const evaluated=await call('Runtime.evaluate',{expression:script,awaitPromise:true,returnByValue:true});
  if(evaluated.exceptionDetails)throw new Error(JSON.stringify(evaluated.exceptionDetails));
  const raw=await call('Runtime.evaluate',{expression:'document.documentElement.dataset.interactionResult',returnByValue:true});
  if(!raw.result.value){const error=await call('Runtime.evaluate',{expression:'document.documentElement.dataset.designError',returnByValue:true});throw new Error(error.result.value||'No interaction report');}
  const report=JSON.parse(raw.result.value),failed=report.checks.filter(check=>!check.pass).map(check=>check.name);
  console.log(theme,width,report.checks.length,'checks; failures:',failed);
  if(failed.length)throw new Error('Design interaction checks failed');reports.push(report);
 }
 await writeFile(join(root,'interactions.json'),JSON.stringify(reports)+'\n');
 if(!process.argv.includes('--check-only')){
 // One browser captures every state with an explicit viewport and measured frame.
 const captureReport=JSON.parse(await readFile(join(root,'verification.json'),'utf8'));
 for(const record of captureReport.captures){
  const theme=record.theme,name=record.file.split('/').at(-1).replace(theme+'-','').replace('.png',''),mode=record.mode;
  const wide=mode==='fullscreen',width=wide?1920:1280,height=wide?1080:960;
  await call('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:1,mobile:false});
  const loaded=once('Page.loadEventFired');
  await call('Page.navigate',{url:pathToFileURL(join(root,'preview.html')).href+'?theme='+theme+'&capture='+name+(wide?'&fullscreen=1':'')+'#'+record.view_index});await loaded;
  const frameWait='new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)))';
  await call('Runtime.evaluate',{expression:frameWait,awaitPromise:true});
  let action='void 0';
  if(mode==='inspector'||mode==='inspector-bottom')action='document.getElementById("right-panel-toggle").click();'+(mode==='inspector-bottom'?'const scroller=document.getElementById("task-inspector-body");scroller.scrollTop=scroller.scrollHeight;':'');
  if(mode==='left-collapsed')action='document.getElementById("sidebar-toggle").click()';
  if(mode==='goal-list')action='i=7;show();showGoalResult("rotation");document.getElementById("goal-result-dialog").close();openWorkspace()';
  if(mode==='delete-warning')action='openWorkspace();document.querySelector(".goal-trash").click()';
  if(mode==='briefing')action='renderTeamBriefing();openSidePanel("team-brief-dialog")';
  if(mode==='chat')action='document.getElementById("coordinator-chat-button").click()';
  if(mode==='profile-edit'||mode==='profile-edit-bottom'){await call('Runtime.evaluate',{expression:'selectRail("agents");'+frameWait,awaitPromise:true});action='openProfileEditor("claude");'+(mode==='profile-edit-bottom'?'const fields=document.querySelector(".profile-editor-fields");fields.scrollTop=fields.scrollHeight;':'');}
  if(mode==='close-confirmation')action='openClosePreview()';
  if(mode==='settings')action='document.getElementById("settings-button").click()';
  if(mode==='sessions'||mode==='sessions-confirm'){await call('Runtime.evaluate',{expression:'selectRail("agents");'+frameWait,awaitPromise:true});action='openSessions("claude")'+(mode==='sessions-confirm'?';document.getElementById("sessions-clean-finished").click()':'');}
  if(mode==='result-file-diff')action='showGoalResult("rotation");const box=document.querySelector("#result-detail .result-section");box.open=true;box.querySelector(".file-diff-toggle").click()';
  if(mode==='goal-title-open')action='document.getElementById("goal-title-toggle").click()';
  if(mode==='mcp')action='openMcp()';
  if(mode==='mcp-add')action='openMcp();openMcpEditor(null)';
  if(mode==='result-next-phase')action='workspaceGoals[0].nextPhase={number:2,total:2};showGoalResult("rotation")';
  if(mode.startsWith('quota-')){
   await call('Runtime.evaluate',{expression:'selectRail("agents");'+frameWait,awaitPromise:true});
   const index=['quota-healthy','quota-error','quota-unsupported'].indexOf(mode);action='document.querySelectorAll(".usage-button")['+index+'].click()';
  }
  const pickerModes={'coordinator-picker':['coordinator','agent',0],'role-picker':['role-1','model',1],'fallback-picker':['role-1','fallback',1],'fallback-model-picker':['role-1','fallbackModel',1]};
  if(pickerModes[mode]){
   const [target,field,view]=pickerModes[mode];await call('Runtime.evaluate',{expression:'i='+view+';show();'+frameWait,awaitPromise:true});action='openAssignmentPicker('+JSON.stringify(target)+','+JSON.stringify(field)+')';
  }
  await call('Runtime.evaluate',{expression:action+';'+frameWait,awaitPromise:true});
  await new Promise(resolve=>setTimeout(resolve,230));
  const state=await call('Runtime.evaluate',{expression:'JSON.stringify({frame:document.querySelector(".win").getBoundingClientRect().toJSON(),view:views[i][1],error:document.documentElement.dataset.designError||null,scrolled:document.querySelector(".win").scrollLeft,dialogs:[...document.querySelectorAll("dialog[open]")].map(d=>d.getBoundingClientRect().toJSON()),quotaOpen:!document.getElementById("usage-popover").hidden,choiceOpen:!selectionMenu.hidden})',returnByValue:true});
  const value=JSON.parse(state.result.value);if(value.error||value.scrolled)throw new Error(name+' invalid frame '+JSON.stringify(value));
  if(mode.startsWith('quota-')&&!value.quotaOpen)throw new Error(name+' quota closed');
  if(pickerModes[mode]&&!value.choiceOpen)throw new Error(name+' choices closed');
  const frame=value.frame;if(frame.x<0||frame.y<0||frame.right>width||frame.bottom>height)throw new Error(name+' frame outside viewport');
  if(value.dialogs.some(d=>d.left<frame.left||d.right>frame.right||d.top<frame.top||d.bottom>frame.bottom))throw new Error(name+' dialog exceeds app frame');
  const image=await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:false,clip:{x:frame.x,y:frame.y,width:frame.width,height:frame.height,scale:1}});
  const bytes=Buffer.from(image.data,'base64');await writeFile(join(root,record.file),bytes);
  Object.assign(record,{frame,view:value.view,viewport:[width,height],sha256:createHash('sha256').update(bytes).digest('hex'),capture_method:'settled-frame DevTools screenshot',checks:[{name:'no script errors or horizontal displacement',pass:true},{name:'frame within explicit viewport',pass:true}]});
  console.log(theme,name,'capture PASS');
 }
 await writeFile(join(root,'verification.json'),JSON.stringify(captureReport)+'\n');
 }

}finally{
 if(socket)socket.close();
 chrome.kill('SIGTERM');
 await new Promise(resolve=>{if(chrome.exitCode!==null)resolve();else{chrome.once('exit',resolve);setTimeout(()=>{chrome.kill('SIGKILL');resolve();},2000).unref();}});
 await rm(folder,{recursive:true,force:true});
}
