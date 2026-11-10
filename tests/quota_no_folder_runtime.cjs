// GitHub #679: a workspace without a project folder explains itself in the usage popup instead of showing a failed lookup.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('quota no-folder skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-quota-folder-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);
 const button="document.querySelector('.startup-profile[data-agent=\"codex\"] .usage-button')";
 for(let i=0;i<100&&!await b.eval(`!!${button}`).catch(()=>false);i++)await wait(100);
 // The card picks the new indicator up on the next poll, so the popup is reopened until it shows the expected state.
 const show=async(indicator,expected)=>{
  await b.eval(`SIM.home.quota_indicators=Object.assign({},SIM.home.quota_indicators,{codex:${JSON.stringify(indicator)}})`);
  let seen=null;for(let i=0;i<120;i++){
   await b.eval(`${button}.click()`);await wait(150);
   seen=await b.eval("({text:document.querySelector('.usage-body')?.textContent||'',alert:!!document.querySelector('.usage-body [role=alert]')})");
   if(expected(seen))return seen;
   await b.eval("document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}))");await wait(150);}
  throw Error('popup never showed the expected state: '+JSON.stringify(seen));};
 const base={status:'unavailable',rings:[],windows:[],budgets:[],observed_at:Math.floor(Date.now()/1000)};
 const folder=await show({...base,error:'Choose a project folder for this workspace first; Claude reports usage only inside a trusted project folder',error_code:'no_project_folder'},s=>!s.alert&&/작업 디렉터리|project folder/.test(s.text));
 if(/CLI가 사용량 응답|did not send a usage response/.test(folder.text))throw Error('no-folder state still says the CLI did not answer: '+folder.text);
 await b.eval("document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}))");await wait(200);
 await show({...base,error:'something else broke'},s=>s.alert);
 console.log('no-folder usage state explained; other errors keep the failure box');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
