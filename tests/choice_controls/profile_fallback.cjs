// #430: the profile editor's fallback model uses the shared anchored choice and accepts a hand-entered model id
// for an agent whose CLI exposes no catalog, registered through the same native catalog request as role fields.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('../design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('profile fallback skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'../..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-profile-fallback-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const preset=`SIM.home.config.profiles.push({id:'aipro',parser:'custom',executable:'not-executed',args:['run','{model}'],permission_mode:'external_sandbox',catalog:null});SIM.reply.catalog=p=>{SIM.home.config.profiles.find(x=>x.id===p.agent).catalog=p.catalog;return {saved:true};};window.profileFixture=true;`;
(async()=>{try{for(const width of [840,1280]){const browser=await launch(chrome,{width,height:960});try{for(const theme of ['light','dark']){
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href+'?run='+theme+width);
 await browser.eval(`PRESET(${JSON.stringify(preset)})`);
 let ready=false;for(let i=0;i<50;i++){try{ready=await browser.eval("!!window.profileFixture && !!document.querySelector('.startup-profile')");}catch{}if(ready)break;await new Promise(r=>setTimeout(r,50));}
 const result=await browser.eval(`(async()=>{
  const checks=[],check=(name,value)=>{if(!value)throw Error(name);checks.push(name);};
  const wait=()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
  document.documentElement.dataset.theme=${JSON.stringify(theme)};await wait();
  const edit=[...document.querySelectorAll('.startup-profile button')].find(b=>b.textContent.trim()===acMessage('startup.tool_edit'));
  check('profile edit button exists',!!edit);edit.click();await wait();
  const agent=document.getElementById('start-fallback-agent'),model=document.getElementById('start-fallback-model');
  check('no alternate agent disables the fallback model',model.disabled);
  agent.value='aipro';agent.dispatchEvent(new Event('change',{bubbles:true}));await wait();
  const trigger=acChoices.buttonFor(model);
  check('fallback model uses the shared anchored choice',!!trigger&&!model.disabled);
  check('catalog-less agent offers manual entry',model.dataset.choiceManual==='true');
  acChoices.open(model);await wait();document.querySelector('#ac-choice-menu [data-value=__manual]').click();
  const input=document.querySelector('#ac-choice-menu input');input.value='internal-model';input.dispatchEvent(new Event('input'));
  document.querySelector('#ac-choice-menu form').dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));
  const deadline=Date.now()+5000;while(model.value!=='internal-model'&&Date.now()<deadline)await new Promise(r=>setTimeout(r,20));
  check('manual fallback model registers through the native catalog request',SIM.log.some(e=>e.kind==='catalog'&&e.p.agent==='aipro'&&e.p.catalog.origin==='manual'));
  check('manual fallback model is applied after registration',model.value==='internal-model');
  agent.value='codex';agent.dispatchEvent(new Event('change',{bubbles:true}));await wait();
  check('changing the fallback agent never keeps an incompatible model',model.value!=='internal-model');
  return checks;
 })()`);
 console.log(theme,width,result.length,'profile fallback checks passed');
}}finally{await browser.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
