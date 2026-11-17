// GitHub #1062: on the review step the summary and its changed-files list stay on screen through background refreshes;
// they used to be hidden by every poll and shown again only after the coordinator reply, so they flickered.
const fs=require('fs'),os=require('os'),path=require('path'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('review summary flicker skipped: no Chrome');process.exit(77);}
(async()=>{const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-review-flicker-'));let b;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(chrome,{width:1280,height:960});
 const diff=['diff --git a/src/a.rs b/src/a.rs','--- a/src/a.rs','+++ b/src/a.rs','@@ -1 +1 @@','-old','+new'].join('\n');
 const setup=`SIM.home.run_available=true;SIM.home.run_state='waiting_approval';SIM.room.report.state='waiting_approval';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'abcdef1234567890'};SIM.reply.evidence=()=>({revision:'x1',report:structuredClone(SIM.room.report),test_output:[{task:'t1',exit_code:0,timed_out:false}],diff:${JSON.stringify(diff)},truncated:false});`;
 const url=pathToFileURL(file).href;await b.goto(url);await b.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(setup)})`);await b.goto(url);
 const result=await b.eval(`(async()=>{const until=async(f,what)=>{const end=Date.now()+(what==="two refreshes"?16000:8000);while(!f()){if(Date.now()>end)throw Error('timed out: '+what);await new Promise(r=>setTimeout(r,30));}};
  await until(()=>document.querySelector('.final-step .session-choice'),'final step');document.querySelector('.final-step .session-choice').click();
  const box=document.getElementById('review-summary');await until(()=>!box.hidden&&box.querySelector('.files'),'summary shown');
  const d=Object.getOwnPropertyDescriptor(HTMLElement.prototype,'hidden');let hides=0;
  Object.defineProperty(box,'hidden',{get(){return d.get.call(this);},set(v){if(v&&!d.get.call(this))hides++;d.set.call(this,v);},configurable:true});
  const polls=()=>SIM.log.filter(x=>x.kind==='room_snapshot').length,start=polls();
  await until(()=>polls()>=start+2,'two refreshes');await new Promise(r=>setTimeout(r,300));
  return {hides,shown:!box.hidden};})()`);
 if(result.hides)throw Error('the review summary was hidden '+result.hides+' time(s) by background refreshes');
 if(!result.shown)throw Error('the review summary is not shown');
 console.log('review summary: stays on screen through background refreshes');
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
