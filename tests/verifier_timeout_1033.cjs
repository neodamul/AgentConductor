// #1033: Settings loads and saves the tool check timeout with the other settings.
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-verifier-timeout-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
browser=await launch(findChrome(),{width:1280});await browser.goto(pathToFileURL(file).href);
const out=await browser.eval(`(async()=>{const q=id=>document.getElementById('start-'+id),check=(n,ok)=>{if(!ok)throw Error(n);return n;},wait=()=>new Promise(r=>setTimeout(r,30));
const until=async(f,n)=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error(n);await wait();}};const done=[];
SIM.reply.update_settings=()=>({repository:'',default_repository:'neodamul/AgentConductor',channel:'stable',verifier_timeout:90});
q('settings-button').click();await until(()=>q('verifier-timeout').value==='90','saved timeout loads');
const field=q('verifier-timeout');done.push(check('the field is a bounded number with a label',field.type==='number'&&field.min==='5'&&field.max==='600'&&field.closest('label').textContent.includes(acMessage('startup.verifier_timeout'))));
field.value='120';q('settings-save').click();await until(()=>SIM.log.some(e=>e.kind==='set_update_settings'),'save request');
done.push(check('saving sends the timeout as a number',SIM.log.find(e=>e.kind==='set_update_settings').p.verifier_timeout===120));return done;})()`);
console.log(out.length,'verifier timeout settings checks passed');
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
