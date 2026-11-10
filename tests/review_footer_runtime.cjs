// GitHub #689: the approval-waiting card shows the summary and a footer (Reject / Request changes / Review changes); the tabs and the
// direction field stay behind it until Review changes or Request changes opens them.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('review footer skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-review-footer-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);await wait(1500);
 const until=async(code,name)=>{for(let i=0;i<150;i++){if(await b.eval(code).catch(()=>false))return;await wait(100);}fail('timed out: '+name);};
 await b.eval("SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.state='waiting_approval';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'abcdef1234567890'};SIM.room.report.tests=[{task:'task-1',exit_code:0,timed_out:false,output:''}];");
 await until("!document.getElementById('start-tab-tasks').disabled",'run steps available');
 await b.eval("document.getElementById('start-tab-tasks').click()");
 // the rail follows into the integration row by itself
 await until("document.querySelector('.final-step .session-choice.active, .final-step .session-choice[aria-current=true], .final-step.active')&&!document.getElementById('review-footer').hidden",'footer shown on the integration row');
 const card="document.querySelector('#run-screen .room-card')";
 const shown=await b.eval(`({only:${card}.classList.contains('review-only'),tabs:getComputedStyle(${card}.querySelector('.view-tabs')).display,composer:getComputedStyle(${card}.querySelector('.composer')).display,banner:document.getElementById('run-banner').hidden,info:document.querySelector('#run-goal-info .goal-info-content').textContent.trim()})`);
 if(!shown.only||shown.tabs!=='none'||shown.composer!=='none')fail('the card still shows tabs or the direction field: '+JSON.stringify(shown));
 if(!shown.banner)fail('the run banner repeats the approval state');if(!shown.info)fail('no integration result line under the title');
 await b.eval("document.getElementById('review-revise').click()");
 const revising=await b.eval(`({only:${card}.classList.contains('review-only'),composer:getComputedStyle(${card}.querySelector('.composer')).display,footer:!document.getElementById('review-footer').hidden})`);
 if(revising.only||revising.composer==='none'||!revising.footer)fail('Request changes did not open the direction field: '+JSON.stringify(revising));
 await b.eval("document.getElementById('review-primary').click()");
 const reviewing=await b.eval(`({only:${card}.classList.contains('review-only'),tabs:getComputedStyle(${card}.querySelector('.view-tabs')).display,review:!document.getElementById('review-view').hidden})`);
 if(reviewing.only||reviewing.tabs==='none'||!reviewing.review)fail('Review changes did not open the evidence: '+JSON.stringify(reviewing));
 // the approved state keeps the footer, and its primary button applies the changes in place of the old status box
 await b.eval("SIM.room.report.state='approved'");
 await until("document.getElementById('review-primary').dataset.mode==='merge'&&document.getElementById('review-revise').hidden&&document.getElementById('run-banner').hidden",'approved footer applies the changes');
 await b.eval("SIM.room.report.state='completed'");
 await until("document.getElementById('review-footer').hidden&&document.getElementById('run-banner').hidden",'completed run shows no status box');
 if(await b.eval("!!document.getElementById('run-path')||!!document.querySelector('.rail-footer')"))fail('the run path or the rail footer is still on the screen');
 console.log('review footer: summary-only card, request changes opens the direction field, review changes opens the evidence');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
