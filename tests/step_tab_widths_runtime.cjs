// #473: stepper tab widths follow their captions. No tab or group keeps free space beyond its own content, so captions of
// similar length get similar widths instead of the old fixed 3:2:1.2 group ratio with equally shared leftovers.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('step tab widths skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-tab-widths-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
(async()=>{try{for(const width of [1280]){const browser=await launch(chrome,{width,height:960});try{
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href+'?run='+width);
 await new Promise(r=>setTimeout(r,1000));
 const result=await browser.eval(`(()=>{
  const checks=[],check=(name,value)=>{if(!value)throw Error(name);checks.push(name);};
  const tabs=[...document.querySelectorAll('.startup-tabs .step-group [role=tab]')];
  check('six step tabs',tabs.length===6);
  const slack=tabs.map(t=>{const c=getComputedStyle(t),kids=[...t.children].map(k=>k.getBoundingClientRect().width),natural=kids.reduce((a,b)=>a+b,0)+parseFloat(c.columnGap||c.gap||0)*(kids.length-1)+parseFloat(c.paddingLeft)+parseFloat(c.paddingRight)+parseFloat(c.borderLeftWidth)+parseFloat(c.borderRightWidth);return [t.textContent.trim(),Math.round(t.getBoundingClientRect().width-natural)];});
  check('no tab keeps free space beyond its content '+JSON.stringify(slack),slack.every(([,s])=>s<=1));
  const groups=[...document.querySelectorAll('.startup-tabs .step-group')];
  const gslack=groups.map(g=>{const kids=[...g.children],sum=kids.reduce((a,k)=>a+k.getBoundingClientRect().width+parseFloat(getComputedStyle(k).marginRight),0),c=getComputedStyle(g),natural=sum+parseFloat(c.columnGap||c.gap||0)*(kids.length-1)+parseFloat(c.paddingLeft)+parseFloat(c.paddingRight);return Math.round(g.getBoundingClientRect().width-natural);});
  check('no group keeps free space beyond its tabs '+JSON.stringify(gslack),gslack.every(s=>s<=1));
  const w=tabs.map(t=>t.getBoundingClientRect().width);
  check('similar captions get similar widths (워크스페이스 vs 검토·병합)',Math.abs(w[0]-w[5])<50);
  return checks;
 })()`);
 console.log(width,result.length,'step tab width checks passed');
}finally{await browser.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
