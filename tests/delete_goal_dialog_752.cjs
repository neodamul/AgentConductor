const assert=require('node:assert/strict'),fs=require('node:fs'),os=require('node:os'),path=require('node:path');
const {execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url'),{findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{
 const chrome=findChrome();assert(chrome,'Chrome is required');const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-752-'));let browser;
 try{
  const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
  for(const width of [840,1280])for(const theme of ['light','dark'])for(const [lang,goal] of [['ko','사내 인증 요구사항과 보안 정책을 검토하고 안전하게 구현한다. '.repeat(24)],['en','Implement the authentication changes and document every security constraint. '.repeat(20)]]){
   browser=await launch(chrome,{width,height:720});await browser.goto(pathToFileURL(file).href);
   const view=await browser.eval(`(()=>{
    document.documentElement.dataset.theme='${theme}';
    const d=document.getElementById('start-delete-goal'),n=document.getElementById('start-delete-goal-name'),p=document.getElementById('start-delete-goal-path');
    n.textContent=${JSON.stringify(goal)};p.textContent='C:/workspace/'.repeat(36)+'goal';d.showModal();
    const identity=document.getElementById('start-delete-goal-identity'),warning=d.querySelector('.danger-text').closest('p'),actions=d.querySelector('.dialog-actions'),r=x=>{const b=x.getBoundingClientRect();return {top:b.top,bottom:b.bottom,height:b.height}};
    return {hasRegion:!!identity,scrollable:identity?identity.scrollHeight>identity.clientHeight:false,identity:identity&&r(identity),warning:r(warning),actions:r(actions),dialog:r(d),height:innerHeight,focus:document.activeElement?.value,name:n.textContent,path:p.textContent};
   })()`);
   assert(view.hasRegion,'Goal and exact path use a bounded, keyboard-scrollable identity region');
   assert(view.scrollable,'long Goal and folder identity remain available inside the bounded region');
   assert(view.warning.top>=view.identity.bottom,'irreversible warning remains outside the scroll region');
   assert(view.actions.bottom<=view.height,'Cancel and delete actions remain in the viewport');
   assert(view.dialog.bottom<=view.height,'confirmation fits the minimum window height');
   assert.equal(view.focus,'cancel','Cancel remains the default focused action');
   await browser.eval("document.querySelector('#start-delete-goal button[value=cancel]').click()");
   assert.equal(await browser.eval("document.getElementById('start-delete-goal').open"),false,'Cancel closes without confirming deletion');
   await browser.close();browser=null;
  }
  console.log('Delete confirmation stays readable across 8 language, theme and width cases.');
 }finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}
})().catch(error=>{console.error(error);process.exitCode=1;});
