// GitHub #1068: a task waiting for its agent's usage limit (or on an unavailable agent) leads with Reassign, outlines the agent
// choice and shows a guide; other tasks keep the quiet assignment controls.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('limited assign guide skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-limited-guide-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
(async()=>{try{const b=await launch(chrome,{width:1440,height:900});try{
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);await wait(1500);
 const until=async(code,name)=>{for(let i=0;i<150;i++){if(await b.eval(code).catch(()=>false))return;await wait(100);}fail('timed out: '+name);};
 await b.eval("SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.tasks[1].status='waiting_availability';SIM.room.report.tasks[1].restriction={source:'usage_limit',observed_at:1};");
 await until("!document.getElementById('start-tab-tasks').disabled",'run steps available');
 await b.eval("document.getElementById('start-tab-tasks').click()");
 await until("document.querySelectorAll('.outline-task').length>1",'task rail');
 const state="({primary:document.getElementById('reassign').classList.contains('primary'),outlined:document.getElementById('agent-select').closest('.inline-control').classList.contains('needs-agent'),guide:!document.getElementById('reassign-guide').hidden&&document.getElementById('reassign-guide').textContent.trim().length>0})";
 await b.eval("document.querySelectorAll('.outline-task')[1].click()");
 await until("document.getElementById('reassign').classList.contains('primary')",'limited task highlighted');
 const limited=await b.eval(state);if(!limited.primary||!limited.outlined||!limited.guide)fail('limited task not highlighted: '+JSON.stringify(limited));
 await b.eval("document.querySelectorAll('.outline-task')[0].click()");
 await until("!document.getElementById('reassign').classList.contains('primary')",'running task quiet');
 const quiet=await b.eval(state);if(quiet.primary||quiet.outlined||quiet.guide)fail('a normal task is highlighted: '+JSON.stringify(quiet));
 // An agent whose account is restricted counts too, before the task itself is marked.
 await b.eval("SIM.room.report.tasks[1].status='pending';delete SIM.room.report.tasks[1].restriction;const a=SIM.room.report.plan.agents.find(x=>x.id===SIM.room.report.plan.tasks[1].agent);SIM.room.report.account_restrictions={[a.quota_group||a.executable]:{source:'usage_limit',observed_at:1}};document.querySelectorAll('.outline-task')[1].click()");
 await until("document.getElementById('reassign').classList.contains('primary')",'restricted account highlighted');
 console.log('limited assign guide: usage-limited and restricted-account tasks lead with Reassign; other tasks stay quiet');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
