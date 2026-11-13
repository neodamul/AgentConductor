const fs=require('fs'),path=require('path'),os=require('os'),assert=require('node:assert/strict'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs'),chrome=findChrome();if(!chrome)process.exit(77);
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-run-recovery-927-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{try{for(const theme of ['light','dark'])for(const width of [840,1280]){
 const b=await launch(chrome,{width,height:960});try{
  await b.goto(pathToFileURL(path.join(dir,'app.html')).href);
  const until=async(code,name)=>{await b.eval("document.dispatchEvent(new Event('visibilitychange'))");for(let i=0;i<150;i++){if(await b.eval(code).catch(()=>false))return;await wait(100);}throw Error(theme+' '+width+': '+name);};
  await b.eval(`document.documentElement.dataset.theme=${JSON.stringify(theme)};SIM.home.run_available=true;SIM.home.run_state='failed';SIM.room.report.state='failed';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.tests=[{task:'task-1',exit_code:1,timed_out:false,output:'assertion failed: <unsafe> token rotation'}];SIM.room.report.error='integration failed';`);
  await until("!document.getElementById('start-tab-tasks').disabled",'run available');
  await b.eval("document.getElementById('start-tab-tasks').click()");
  await until("document.getElementById('run-pill').textContent.includes('실패')",'failed state loaded');
  await until("document.getElementById('start-tab-verify').getAttribute('aria-selected')==='true'",'failed integration stays on Verify');
  assert.equal(await b.eval("!!document.getElementById('run-banner')"),false,'no banner in failed state');
  await until("document.getElementById('run-error')?.textContent.includes('<unsafe> token rotation')&&!document.getElementById('fix-integration').hidden",'failure evidence and repair');
  if(process.env.AC_927_CAPTURE){fs.mkdirSync(process.env.AC_927_CAPTURE,{recursive:true});fs.writeFileSync(path.join(process.env.AC_927_CAPTURE,theme+'-'+width+'.png'),await b.shot());}
  assert(await b.eval("!!document.querySelector('.room-card #run-error')&&!!document.querySelector('.room-card #fix-integration')&&!document.querySelector('unsafe')"));
  await b.eval("document.getElementById('fix-integration').click()");
  assert(await b.eval("document.getElementById('feedback').value.includes('<unsafe> token rotation')"),'repair drafts the failed output');
  await b.eval("SIM.room.report.tests=[];SIM.room.report.error='out-of-scope change: outside.txt';SIM.reply.scope_failure=()=>({offending:['outside.txt'],stray:[],tasks:SIM.plan.tasks});");
  await until("!document.getElementById('scope-review').hidden",'scope action retained');
  await b.eval("document.getElementById('scope-review').click()");
  await until("document.getElementById('scope-dialog').open",'scope review opens');
  await b.eval("document.getElementById('scope-dialog').close();SIM.room.report.state='running';SIM.room.report.error=null;SIM.room.report.tasks[0].status='running';SIM.room.runner='absent';SIM.room.report.direct_workspace=true;SIM.room.report.plan.workspace_mode='shared';SIM.room.owned_changes=['src/a'];SIM.room.interruptions=[{at:Date.now(),status:'interrupted'}];");
  await until("document.getElementById('run-attention')?.textContent&& !document.getElementById('owned-file-warning').hidden&&!document.getElementById('run-interruptions').hidden",'attention and safety warnings retained');
  assert(await b.eval("!document.getElementById('run-banner')&&!document.getElementById('run-context').hidden"));
  await b.eval("SIM.room.report.state='waiting_approval';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'abcdef1234567890'};SIM.room.report.tests=[{task:'task-1',exit_code:0,timed_out:false,output:''}];");
  await until("!document.getElementById('review-footer').hidden",'approval footer retained');
  await b.eval("SIM.room.report.state='approved'");
  await until("document.getElementById('review-primary').dataset.mode==='merge'",'merge action retained');
  await b.eval("SIM.room.report.state='completed'");
  await until("document.getElementById('review-primary').dataset.mode==='result'",'result action retained');
  assert.equal(await b.eval("!!document.getElementById('run-banner')"),false);
  console.log('run recovery927',theme,width,'passed');
 }finally{await b.close();}
}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
