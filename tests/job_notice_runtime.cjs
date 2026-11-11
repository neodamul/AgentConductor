// GitHub #820: the finished-job notice names the action in the interface language, and a job that had already
// finished when the window opened is not announced again.
const fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('job notice skipped: no Chrome');return;}const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-job-notice-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
 browser=await launch(chrome,{width:1280});
 const preset="SIM.home.job={id:'finished-before-launch',kind:'start',state:'completed',exit_code:0};";
 await browser.goto(pathToFileURL(file).href);await browser.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(preset)})`);await browser.goto(pathToFileURL(file).href);
 const checked=await browser.eval(`(async()=>{
  const q=id=>document.getElementById(id),checks=[],check=(name,ok)=>{if(!ok)throw Error(name+' notice='+JSON.stringify(q('start-notice').textContent));checks.push(name);};
  const until=async(f,name)=>{const end=Date.now()+15000;while(!f()){if(Date.now()>end)throw Error(name);await wait(30);}};
  await until(()=>SIM.log.filter(e=>e.kind==='snapshot').length>=3,'snapshots');
  check('a job finished before launch is not announced',!q('start-notice').textContent.includes(acMessage('state.start')));
  SIM.home.job={id:'finished-now',kind:'mcp',state:'completed',exit_code:0};SIM.home.revision='r-job';
  await until(()=>q('start-notice').textContent.length>0,'notice for the new job');
  check('the notice uses the localized action name',q('start-notice').textContent===acMessage('startup.completed_93d6b7',{value1:acMessage('state.mcp')})&&!q('start-notice').textContent.includes('mcp '));
  return checks;
 })()`);
 console.log(checked.length,'job notice checks passed');
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
