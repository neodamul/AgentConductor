// #857: only confirmed newer releases advertise themselves in the shared titlebar.
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome)throw Error('Chrome is required');const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-update-indicator-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
for(const width of [840,1280]){browser=await launch(chrome,{width});for(const theme of ['light','dark']){
 await browser.goto(pathToFileURL(file).href);await browser.eval(`sessionStorage.setItem('SIM_PRESET',"SIM.hang.check_update=true;window.updateFixture={phase:'idle',current:'1.0.0',newer:false};SIM.reply.check_update=SIM.reply.update_status=()=>window.updateFixture;")`);await browser.goto(pathToFileURL(file).href);
 const checks=await browser.eval(`(async()=>{
 const q=id=>document.getElementById('start-'+id),out=[],check=(n,ok)=>{if(!ok)throw Error(n);out.push(n);},wait=()=>new Promise(r=>setTimeout(r,30));
 const until=async(f,n)=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error(n);await wait();}};
 document.documentElement.dataset.theme='${theme}';
 check('initial unknown update state is hidden',q('update-button').hidden&&q('update-button').getClientRects().length===0);
 SIM.hang.check_update=false;SIM.release('check_update');await wait();
 const manual=async state=>{window.updateFixture=state;q('app-menu').click();await until(()=>!q('workspace-list').hidden,'menu');const b=[...q('workspace-list').querySelectorAll('button')].find(b=>b.textContent.includes(acMessage('startup.check_for_updates')));check('manual update check remains in menu',!!b);b.click();await until(()=>q('update').open,'manual update dialog');const expected=state.phase==='checking'?acMessage('startup.update_checking'):state.newer?acMessage('startup.update_newer'):state.error?acMessage('startup.update_failed',{value1:state.error}):acMessage('startup.update_up_to_date');await until(()=>q('update-title').textContent.startsWith(expected),'update state rendered');};
 await manual({phase:'idle',current:'1.0.0',newer:false});check('up-to-date indicator stays hidden',q('update-button').hidden);q('update').close();
 await manual({phase:'idle',current:'1.0.0',latest:'1.1.0',newer:true});check('newer release reveals actionable indicator',!q('update-button').hidden&&q('update-button').getAttribute('aria-label').includes('1.1.0'));q('update').close();
 q('update-button').click();await until(()=>q('update').open,'indicator opens dialog');q('update').close();
 await manual({phase:'idle',current:'1.0.0',newer:false,error:'offline'});check('failed check without a newer release hides indicator',q('update-button').hidden);q('update').close();
 await manual({phase:'checking',current:'1.0.0',newer:false});check('checking without a known update stays hidden',q('update-button').hidden);q('update').close();
 await manual({phase:'installing',current:'1.0.0',latest:'1.1.0',newer:true});check('known update stays visible during installation',!q('update-button').hidden);q('update').close();
 window.updateFixture={phase:'idle',current:'1.0.0',latest:'1.1.0',newer:true};SIM.home.run_available=true;SIM.home.run_state='running';SIM.home.revision='update-room';await until(()=>!q('tab-tasks').disabled,'Room available');q('tab-tasks').click();await until(()=>document.body.classList.contains('on-run'),'Room opens');check('shared indicator survives Room navigation',!q('update-button').hidden&&document.querySelector('header.titlebar').contains(q('update-button')));
 await manual({phase:'idle',current:'1.1.0',newer:false});check('Room also hides an up-to-date indicator',document.body.classList.contains('on-run')&&q('update-button').hidden);q('update').close();return out;
 })()`);console.log(theme,width,checks.length,'update indicator checks passed');
}await browser.close();browser=null;}
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
