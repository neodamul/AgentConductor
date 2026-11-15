// #999: Resume stays off while any task of the run is still running, even when the run state is `ready`.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('Resume running skipped: no Chrome');return;}const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-999-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);browser=await launch(chrome,{width:1280});
 await browser.goto(pathToFileURL(file).href);
 const states=await browser.eval(`(async()=>{const q=id=>document.getElementById(id),wait=ms=>new Promise(r=>setTimeout(r,ms));
  const until=async(f,name)=>{const end=Date.now()+15000;while(!f()){if(Date.now()>end)throw Error(name);await wait(30);}};
  SIM.room.report.state='ready';SIM.room.report.tasks[0].status='running';SIM.home.run_available=true;SIM.home.run_state='ready';
  await until(()=>!q('start-tab-tasks').disabled,'task tab');q('start-tab-tasks').click();await until(()=>document.body.classList.contains('on-run'),'agent room');await wait(1500);
  const running=q('resume').disabled;
  for(const t of SIM.room.report.tasks)if(t.status==='running')t.status='pending';SIM.room.revision='x2';await wait(1500);
  return {running,idle:q('resume').disabled};})()`);
 assert.equal(states.running,true,'Resume must be disabled while a task is running');
 assert.equal(states.idle,false,'Resume returns once no task is running');
 console.log('Resume running: ok');
}finally{await browser?.close?.();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exit(1);});
