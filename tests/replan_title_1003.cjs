// #1003: while Phase 3 is planned after Phase 2, the pending title says Phase 2 → Phase 3 even before the proposal moves on.
const fs=require('fs'),os=require('os'),path=require('path'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('Replan title skipped: no Chrome');return;}const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-replan-1003-'));let b;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(chrome,{width:1280,height:960});
const setup=`SIM.home.proposal=SIM.proposal('ready');SIM.home.proposal.slug='g';SIM.home.proposal.slug_phase='p2';SIM.home.proposal.replan={unchanged:false,completed_phases:[{phase:'p1'}],changes:[]};SIM.home.slugs=[{slug:'g',active:true,goal:SIM.home.config.goal,plan:{phases:[]},progress:{current_phase:'p2',phases:[1,2,3].map(n=>({id:'p'+n,name:'Phase '+n,stage:n===1?'completed':'task',task:[],completed:[],verification:[]}))}}];`;
for(const lang of ['ko','en']){
 await b.goto(pathToFileURL(file).href);await b.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(setup+`SIM.home.language='${lang}';`)})`);await b.goto(pathToFileURL(file).href);
 // Phase 2 finished; the plan job for Phase 3 runs while slug_phase and current_phase still name Phase 2.
 await b.eval(`(async()=>{while(!document.querySelector('#start-tasks .startup-task'))await new Promise(r=>setTimeout(r,30));AC_I18N.set('${lang}');document.getElementById('start-tab-review').click();SIM.home.proposal={...SIM.home.proposal,phase:'planning',directory:'next-phase',digest:null,plan:null,replan:null};SIM.home.slugs[0].progress.phases[1].stage='completed';SIM.home.job={id:'replan-three',kind:'plan',state:'running',started_ms:Date.now()};})()`);
 await b.eval(`(async()=>{const end=Date.now()+6500,head=()=>document.querySelector('#start-panel-review .step-title').textContent;while(!document.getElementById('start-panel-review').classList.contains('plan-pending')){if(Date.now()>end)throw Error('not pending: '+head());await new Promise(r=>setTimeout(r,30));}const h=head();if(!h.includes('Phase 2')||!h.includes('Phase 3')||h.includes('Phase 1'))throw Error('Stale heading: '+h);})()`);
 console.log(lang,'pending title names Phase 2 → Phase 3');
}
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
