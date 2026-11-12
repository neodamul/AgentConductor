// #862: one Help action and one shared repository setting on Workspace and Room.
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome)throw Error('Chrome is required');const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-bug-report-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
for(const width of [840,1280]){browser=await launch(chrome,{width});for(const theme of ['light','dark']){
 await browser.goto(pathToFileURL(file).href);
 const checks=await browser.eval(`(async()=>{
 const q=id=>document.getElementById('start-'+id),out=[],check=(n,ok)=>{if(!ok)throw Error(n);out.push(n);},wait=()=>new Promise(r=>setTimeout(r,30));
 const until=async(f,n)=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error(n);await wait();}};
 document.documentElement.dataset.theme='${theme}';SIM.reply.report_bug=()=>({opened:true,repository:'team/internal'});
 const openHelp=async()=>{q('app-menu').click();await until(()=>!q('workspace-list').hidden,'menu opens');const group=[...q('workspace-list').querySelectorAll('.menu-parent')].find(b=>b.textContent.trim()===acMessage('startup.menu_help'));group.click();const b=[...group.nextElementSibling.querySelectorAll('button')].find(b=>b.textContent.trim()===acMessage('startup.report_bug'));check('Help has a visible Report a bug action',!!b&&b.getClientRects().length>0);return b;};
 (await openHelp()).click();await until(()=>SIM.log.some(e=>e.kind==='report_bug'),'native report request');await until(()=>q('notice').textContent.includes('team/internal'),'browser result notice');
 const request=SIM.log.find(e=>e.kind==='report_bug').p;check('the page sends no URL or diagnostic content',JSON.stringify(request)===JSON.stringify({kind:'report_bug',open:true}));check('reporting closes the menu and keeps Workspace',q('workspace-list').hidden&&!document.body.classList.contains('on-run'));
 SIM.reply.report_bug=new Error('browser refused');(await openHelp()).click();await until(()=>q('notice').textContent.includes('browser refused'),'native launch error');check('browser launch failures are visible',q('notice').textContent.includes('browser refused'));
 SIM.reply.report_bug=()=>({opened:true,repository:'team/internal'});SIM.home.run_available=true;SIM.home.run_state='running';SIM.home.revision='bug-room';await until(()=>!q('tab-tasks').disabled,'Room ready');q('tab-tasks').click();await until(()=>document.body.classList.contains('on-run'),'Room opens');
 const before=SIM.log.filter(e=>e.kind==='report_bug').length;(await openHelp()).click();await until(()=>SIM.log.filter(e=>e.kind==='report_bug').length>before,'Room report');check('Room report preserves the active run',document.body.classList.contains('on-run'));
 SIM.reply.update_settings=()=>({repository:'https://ghe.corp/team/tool',default_repository:'neodamul/AgentConductor',channel:'stable'});q('settings-button').click();await until(()=>q('update-repository').value==='https://ghe.corp/team/tool','shared repository loaded');check('settings explains the shared report/update repository',q('update-repository').closest('label').textContent.includes(acMessage('startup.update_repository'))&&/bug|버그/i.test(acMessage('startup.update_source_hint')));q('settings').close('cancel');return out;
 })()`);console.log(theme,width,checks.length,'bug report menu checks passed');
}await browser.close();browser=null;}
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
