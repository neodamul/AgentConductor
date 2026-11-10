// #430: the Room's task inspector model is an anchored choice; when the agent's CLI exposes no catalog (or the operator
// registered one by hand) it accepts a hand-entered id, sent as one explicit register-and-apply model action.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('../design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('inspector model skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'../..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-inspector-model-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const preset=`SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.state='ready';SIM.room.report.plan.agents.forEach(a=>a.catalog=null);SIM.room.report.tasks.forEach(t=>t.status='pending');SIM.reply.action=p=>({accepted:true,kind:p.action.kind,message:'ok'});window.inspectorFixture=true;`;
(async()=>{try{for(const width of [840,1280]){const browser=await launch(chrome,{width,height:960});try{for(const theme of ['light','dark']){
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href+'?run='+theme+width);
 await browser.eval("window.__beforePreset=true");
 await browser.eval(`PRESET(${JSON.stringify(preset)})`);
 let ready=false;for(let i=0;i<50;i++){try{ready=await browser.eval("!window.__beforePreset && !!window.inspectorFixture && !!document.getElementById('start-tab-tasks')");}catch{}if(ready)break;await new Promise(r=>setTimeout(r,50));}
 const result=await browser.eval(`(async()=>{
  const checks=[],check=(name,value)=>{if(!value)throw Error(name);checks.push(name);};
  const wait=()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
  document.documentElement.dataset.theme=${JSON.stringify(theme)};await wait();
  const until=Date.now()+20000;while(document.getElementById('start-tab-tasks').disabled&&Date.now()<until)await new Promise(r=>setTimeout(r,30));
  document.getElementById('start-tab-tasks').click();
  const end=Date.now()+20000;while(!document.body.classList.contains('on-run')&&Date.now()<end)await new Promise(r=>setTimeout(r,30));
  let model=null;const found=Date.now()+20000;while(!(model=document.getElementById('model-select'))&&Date.now()<found)await new Promise(r=>setTimeout(r,30));
  if(!model)throw Error('the task inspector never rendered #model-select');const ready=Date.now()+20000;while(model.dataset.choiceManual!=='true'&&Date.now()<ready)await new Promise(r=>setTimeout(r,30));
  check('inspector model is an anchored choice',!!acChoices.buttonFor(model));
  check('catalog-less agent offers manual entry',model.dataset.choiceManual==='true');
  acChoices.open(model);await wait();document.querySelector('#ac-choice-menu [data-value=__manual]').click();
  const input=document.querySelector('#ac-choice-menu input');input.value='operator/own';input.dispatchEvent(new Event('input'));
  document.querySelector('#ac-choice-menu form').dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));
  const deadline=Date.now()+20000;while(!SIM.log.some(e=>e.kind==='action'&&e.p.action.kind==='model')&&Date.now()<deadline)await new Promise(r=>setTimeout(r,20));
  const sent=SIM.log.filter(e=>e.kind==='action'&&e.p.action.kind==='model').map(e=>e.p.action);
  check('manual id is sent as an explicit register-and-apply action',sent.length===1&&sent[0].model==='operator/own'&&sent[0].register===true&&sent[0].task===document.getElementById('selected-task').textContent.split(' · ')[1]);
  return checks;
 })()`);
 console.log(theme,width,result.length,'inspector model checks passed');
}}finally{await browser.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
