// GitHub #1053: a waiting task's tag has one colour: the run tab's status pill matches the task rail's badge.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('pending tone skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-pending-tone-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
(async()=>{try{const b=await launch(chrome,{width:1440,height:900});try{
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);await wait(1500);
 const until=async(code,name)=>{for(let i=0;i<150;i++){if(await b.eval(code).catch(()=>false))return;await wait(100);}fail('timed out: '+name);};
 await b.eval("SIM.home.run_available=true;SIM.home.run_state='running';");
 await until("!document.getElementById('start-tab-tasks').disabled",'run steps available');
 await b.eval("document.getElementById('start-tab-tasks').click()");
 await until("document.querySelectorAll('.outline-task').length>1",'task rail');
 await b.eval("document.querySelectorAll('.outline-task')[1].click()");
 await until("document.getElementById('task-status').classList.contains('tone-idle')",'pending task selected');
 const colors=await b.eval("(()=>{const c=e=>{const s=getComputedStyle(e);return s.color+' '+s.backgroundColor;};return {rail:c(document.querySelectorAll('.outline-task')[1].querySelector('.outline-state-badge')),pill:c(document.getElementById('task-status'))};})()");
 if(colors.rail!==colors.pill)fail('waiting tag colours differ: '+JSON.stringify(colors));
 console.log('pending tone: the run tab pill matches the task rail badge');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
