// #1001: while a run is active again (automatic repair), the previous error and failed verification are not shown as an operator-action error.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {launch,findChrome}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('Stale error skipped: no Chrome');return;}let b;const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-1001-'));
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(chrome,{width:1280,height:900});
for(const theme of ['light','dark']){
 await b.goto(pathToFileURL(file).href);
 await b.eval(`{document.documentElement.dataset.theme='${theme}';SIM.home.run_available=true;SIM.home.run_state='running';const r=SIM.room.report;r.state='running';r.tasks[0].status='running';r.error='integration test failed: task-1 (expected success)';r.tests=[{task:r.tasks[0].id,exit_code:1,timed_out:false,passed:false}];}`);
 const until=async(expr)=>{for(let i=0;i<150;i++){if(await b.eval(expr))return;await new Promise(r=>setTimeout(r,50));}throw Error('UI did not settle: '+expr);};
 await until("!document.getElementById('start-tab-tasks').disabled");
 await b.eval("document.getElementById('start-tab-tasks').click()");
 await until("document.body.classList.contains('on-run')");await new Promise(r=>setTimeout(r,1200));
 assert.equal(await b.eval("document.getElementById('run-error').hidden"),true,'stale error shown while the run is active');
 assert.equal(await b.eval("document.getElementById('fix-integration').hidden"),true,'fix offered while the run is active');
 assert.equal(await b.eval("document.getElementById('resume').disabled"),true,'Resume enabled while the run is active');
 // Once the run has failed again and nothing runs, the error is the operator's to act on.
 await b.eval("{const r=SIM.room.report;r.state='failed';r.tasks.forEach(t=>t.status='completed');SIM.room.revision='failed-again';}");
 await until("!document.getElementById('run-error').hidden");
 console.log(theme,'stale error hidden while active, shown after failure');
}
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
