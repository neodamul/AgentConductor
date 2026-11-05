// #473: natural-width tabs shrink only when necessary and retain full localized titles.
const assert=require('node:assert/strict'),fs=require('node:fs'),os=require('node:os'),path=require('node:path');
const {execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const root=process.cwd(),{findChrome,launch}=require(path.join(root,'tests/design_parity/cdp.cjs'));
const chrome=findChrome();if(!chrome){console.log('step tabs skipped: no Chrome');process.exit(77);}
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-step-tabs-'));
execFileSync(process.execPath,[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
(async()=>{try{for(const width of [650,840,1280]){const browser=await launch(chrome,{width,height:960});try{
 await browser.goto(pathToFileURL(path.join(root,'docs/design/approved-goal-ui/preview.html')).href+'?theme=light#0');
 const mock=await browser.eval(`(()=>{let s=document.createElement('style');s.textContent='.win{width:calc(100vw + 2px)!important;max-width:none!important}';document.head.append(s);return [...document.querySelectorAll('.tabs .step-group')].map(e=>e.getBoundingClientRect().width)})()`);
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href);
 const app=await browser.eval(`(()=>{let p=document.querySelector('#start-step-tabs'),g=[...p.querySelectorAll('.step-group')];return {available:p.clientWidth,used:g.reduce((n,e)=>n+e.getBoundingClientRect().width,0)+parseFloat(getComputedStyle(p).gap)*2,groups:g.map(e=>e.getBoundingClientRect().width)}})()`);
 assert(app.used<=app.available+1,'app groups fit within navigation');
 mock.forEach((v,i)=>assert(Math.abs(v-app.groups[i])<1,`mock group ${i}: ${v} vs ${app.groups[i]} at ${width}px`));
 for(const lang of ['ko','en']){
  const tabs=await browser.eval(`(()=>{AC_I18N.set('${lang}');return [...document.querySelectorAll('#start-step-tabs [role=tab]')].map(e=>{const l=e.querySelector('span:last-child');return {name:l.textContent.trim(),title:e.title,truncated:l.scrollWidth>l.clientWidth+1}})})()`);
  assert.equal(tabs.length,6);
  assert(tabs.every(t=>t.name&&t.title===t.name),`${lang} requires per-tab localized full-label titles: ${JSON.stringify(tabs)}`);
  if(width===650)assert(tabs.some(t=>t.truncated),'narrow layout exercises ellipsis');
 }
 console.log(`step tabs at ${width}px: reference geometry and localized titles pass`);
}finally{await browser.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
