// #409: role fields use reviewed task policies and the operator catalog request.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('../design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('role choices skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'../..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-role-choice-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const preset=`SIM.home.config.profiles[0].catalog.models=['shared','opus'];SIM.home.config.profiles[1].catalog.models=['shared','gpt'];SIM.home.config.profiles[1].fallback={agent:'aipro',model:null};SIM.home.config.profiles.push({id:'aipro',parser:'custom',executable:'not-executed',args:['run','{model}'],permission_mode:'external_sandbox',catalog:null});SIM.home.config.roles={development:[{agent:'claude',model:'shared',fallback:{agent:'codex',model:'gpt'}}],testing:[{agent:'codex',fallback:false}]};SIM.reply.catalog=p=>{SIM.home.config.profiles.find(x=>x.id===p.agent).catalog=p.catalog;return {saved:true};};window.roleFixture=true;`;
(async()=>{try{for(const width of [840,1280]){const browser=await launch(chrome,{width,height:960});try{for(const theme of ['light','dark']){
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href+'?run='+theme+width);
 await browser.eval(`PRESET(${JSON.stringify(preset)})`);
 let ready=false;for(let i=0;i<50;i++){try{ready=await browser.eval("!!window.roleFixture && !!document.querySelector('tr[data-role=development]')");}catch{}if(ready)break;await new Promise(r=>setTimeout(r,50));}
 const result=await browser.eval(`(async()=>{
  const checks=[],check=(name,value)=>{if(!value)throw Error(name);checks.push(name);};
  const wait=()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
  const field=(role,name)=>document.querySelector('tr[data-role="'+role+'"] [data-role-field="'+name+'"]');
  const change=(s,value)=>{s.value=value;s.dispatchEvent(new Event('change',{bubbles:true}));};
  document.documentElement.dataset.theme=${JSON.stringify(theme)};document.getElementById('start-tab-team').click();await wait();
  check('dedicated role renderer exists',!!window.acRoleChoices);
  const a=field('development','agent'),m=field('development','model'),f=field('development','fallback'),fm=field('development','fallback-model');
  check('saved role model and fallback restored',a.value==='claude'&&m.value==='shared'&&f.value==='codex'&&fm.value==='gpt');
  check('primary is absent from its own fallback options',![...f.options].some(o=>o.value==='claude'));
  check('muted arrow is eighteen pixels',Math.abs(document.querySelector('tr[data-role=development] .role-fallback-arrow svg').getBoundingClientRect().width-18)<.1);
  change(a,'codex');await wait();check('agent change resets even a shared model',m.value==='');
  check('self fallback cleared and its model disabled',f.value===''&&fm.disabled);
  change(f,'__inherit');await wait();check('profile default is inherited without role override',!Object.hasOwn(acRoleChoices.read(document.getElementById('start-roles')).development[0],'fallback'));
  check('inherited fallback profile is aipro',fm.dataset.modelAgent==='aipro');
  const coordinator=document.getElementById('start-coordinator');change(coordinator,'aipro');await wait();check('unsupported catalog keeps coordinator default enabled',!document.getElementById('start-coordinator-model').disabled); 
  acChoices.open(fm);await wait();document.querySelector('#ac-choice-menu [data-value=__manual]').click();
  const input=document.querySelector('#ac-choice-menu input');input.value='internal-model';input.dispatchEvent(new Event('input'));document.querySelector('#ac-choice-menu form').dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));
  const deadline=Date.now()+5000;while(fm.value!=='internal-model'&&Date.now()<deadline)await new Promise(r=>setTimeout(r,20));
  check('manual model registers through native catalog request',SIM.log.some(e=>e.kind==='catalog'&&e.p.agent==='aipro'&&e.p.catalog.origin==='manual'));
  check('manual model applied after registration',fm.value==='internal-model');
  const rules=acRoleChoices.read(document.getElementById('start-roles'));
  check('editing inherited fallback model makes a role override',rules.development[0].fallback.agent==='aipro'&&rules.development[0].fallback.model==='internal-model');
  check('other role explicit none remains unchanged',rules.testing[0].fallback===false);
  change(f,'');await wait();check('none disables and clears fallback model',fm.disabled&&fm.value==='');
  check('coordinator default is available without a catalog',!document.getElementById('start-coordinator-model').disabled);
  const registration=document.getElementById('start-coordinator-model').acManualPick('race-model').then(()=>false,()=>true);change(coordinator,'codex');check('late manual registration cannot select for another coordinator',await registration);await wait();
  const table=document.createElement('table');table.innerHTML='<thead><tr><th>Role</th><th>Agent</th><th>Model</th><th>Fallback</th><th>Model</th></tr></thead><tbody></tbody>';document.body.append(table);
  const ps=[{id:'a',executable:'a',fallback:{agent:'b',model:'bm'}},{id:'b',executable:'b',catalog:{models:['bm']}},{id:'c',executable:'c',catalog:{models:['cm']}},{id:'__inherit',executable:'special'}];
  let rs={development:[{agent:'a'}],testing:[{agent:'missing-agent'}]};
  acRoleChoices.render({table,getProfiles:()=>ps,getRules:()=>rs,labels:{development:'Development',testing:'Testing'},onChange:()=>{},modelHint:()=>'',registerModel:async()=>{throw Error('not used');}});
  const fs=table.querySelector('[data-role-field=fallback]'),ms=table.querySelector('[data-role-field=fallback-model]');
  check('missing saved primary remains visible for reassignment',table.querySelector('tr[data-role=testing] [data-role-field=agent]').value==='missing-agent'&&acRoleChoices.read(table).testing[0].agent==='missing-agent');
  check('inherit token cannot collide with registered agent id',fs.value!=='__inherit'&&[...fs.options].some(o=>o.value==='__inherit'));
  ps[0].fallback={agent:'c',model:'cm'};acRoleChoices.refresh(document.activeElement);
  check('changed profile default updates inherited label and model',fs.options[0].textContent.includes('c')&&ms.dataset.modelAgent==='c'&&ms.value==='cm');
  delete ps[0].fallback;acRoleChoices.refresh(document.activeElement);check('removed profile default disables inherited model',ms.disabled&&ms.value==='');table.remove();
  return checks;
 })()`);
 console.log(theme,width,result.length,'role checks passed');
}}finally{await browser.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
