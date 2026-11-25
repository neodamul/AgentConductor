// #960: each new run starts on Activity; polling the same run preserves the chosen tab.
const fs=require('fs'),os=require('os'),path=require('path'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-run-tab-960-'));let b;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(findChrome(),{width:1280,height:960});
const setup=`SIM.home.run_available=true;SIM.home.run_state='running';SIM.reply.evidence=()=>({report:structuredClone(SIM.room.report),test_output:[],diff:'fixture diff',truncated:false});`;
await b.goto(pathToFileURL(file).href);await b.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(setup)})`);await b.goto(pathToFileURL(file).href);
await b.eval(`(async()=>{window.untilTab=async f=>{const end=Date.now()+7000;while(!f()){if(Date.now()>end)throw Error('Tab state did not update');await new Promise(r=>setTimeout(r,30));}};window.tabIs=name=>document.querySelector('[data-tab="'+name+'"]').getAttribute('aria-selected')==='true'&&!document.getElementById(name+'-view').hidden;await untilTab(()=>document.body.classList.contains('on-run')&&document.querySelector('[data-tab="review"]').onclick);document.querySelector('[data-tab="review"]').click();await untilTab(()=>tabIs('review'));window.snapshotRuns=[];const observe=acSessions.observe;acSessions.observe=next=>{if(next.run_dir)snapshotRuns.push(next.run_dir);observe(next);};await new Promise(r=>setTimeout(r,2300));if(!tabIs('review'))throw Error('Same-run poll reset user tab');SIM.room.run_dir='/tmp/run-b';await untilTab(()=>snapshotRuns.includes('/tmp/run-b'));if(!tabIs('activity'))throw Error('New run inherited review tab');})()`);
// A new Room closure reuses DOM from the old one, just like next-Phase open_run.
await b.eval(`(async()=>{document.querySelector('[data-tab="review"]').click();await untilTab(()=>tabIs('review'));window.stopRoom();SIM.room.run_dir='/tmp/run-c';window.startRoom();await untilTab(()=>snapshotRuns.includes('/tmp/run-c'));if(!tabIs('activity'))throw Error('New Room retained old DOM selection');SIM.hang.evidence=true;document.querySelector('[data-tab="review"]').click();await untilTab(()=>SIM.waiters.evidence?.length);SIM.room.run_dir='/tmp/run-d';await untilTab(()=>snapshotRuns.includes('/tmp/run-d'));SIM.release('evidence');await new Promise(r=>setTimeout(r,100));if(!tabIs('activity'))throw Error('Late previous-run evidence changed new run tab');})()`);
await b.eval(`(async()=>{SIM.hang.evidence=false;SIM.room.report.state='waiting_approval';SIM.room.report.candidate={commit:'abcdef1234567890'};SIM.room.report.tasks.forEach(t=>t.status='completed');await new Promise(r=>setTimeout(r,2200));document.querySelector('.final-step .session-choice').click();await untilTab(()=>!document.getElementById('review-summary').hidden);if(!tabIs('activity'))throw Error('Background review summary changed chosen tab');})()`);
// #1096: the shared numbered tabs meet the Room card in both reference sizes and themes.
for (const width of [840, 1280]) {
  await b.viewport(width, 960);
  for (const theme of ['light', 'dark']) {
    const gap = await b.eval(`(() => {
      document.documentElement.dataset.theme = '${theme}';
      const stepper = document.getElementById('start-step-tabs').getBoundingClientRect();
      const card = document.querySelector('#run-screen .room-card').getBoundingClientRect();
      if (!stepper.height || !card.height) throw Error('Room stepper and card must be visible');
      return card.top - stepper.bottom;
    })()`);
    if (Math.abs(gap) > 0.5) throw Error(`Room stepper/card gap: ${gap}px (${width}px ${theme})`);
  }
}
console.log('Room #1096: stepper and card meet in both themes at 840px and 1280px');
console.log('Room #960: same-run selection, changed run, recreated Room and late evidence passed');
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
