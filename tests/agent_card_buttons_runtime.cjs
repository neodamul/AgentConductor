// GitHub #654: in English the agent card's tool row (Edit, Models, remove, up, down) stays on one line at both reference widths.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('agent card buttons skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-agent-card-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
(async()=>{try{for(const width of [840,1280]){const b=await launch(chrome,{width,height:900});try{
 const url=pathToFileURL(path.join(dir,'app.html')).href;
 await b.goto(url);await b.eval("sessionStorage.setItem('SIM_PRESET',\"SIM.home.language='en';\")");await b.goto(url);
 let ready=false;for(let i=0;i<100;i++){try{ready=await b.eval("document.querySelectorAll('.profile-tools').length>0");}catch{}if(ready)break;await new Promise(r=>setTimeout(r,50));}
 await b.eval("AC_I18N.set('en')");await new Promise(r=>setTimeout(r,500));
 for(const theme of ['light','dark']){
  const rows=await b.eval(`document.documentElement.dataset.theme='${theme}';[...document.querySelectorAll('.profile-tools')].map(t=>{const q=t.getBoundingClientRect(),k=[...t.children].map(c=>c.getBoundingClientRect());return {h:q.height,labels:t.textContent.trim(),tops:new Set(k.map(r=>Math.round(r.top))).size,right:Math.max(...k.map(r=>r.right))-q.right}})`);
  if(!rows.length)throw Error('no agent cards');
  // A narrower card (a longer name, a larger system font) must not push the last button to a second line either.
  rows.push(...await b.eval(`document.querySelectorAll('.startup-profile').forEach(c=>c.style.width='150px');[...document.querySelectorAll('.profile-tools')].map(t=>{const q=t.getBoundingClientRect(),k=[...t.children].map(c=>c.getBoundingClientRect());return {h:q.height,labels:t.textContent.trim()+' @150',tops:new Set(k.map(r=>Math.round(r.top))).size,right:Math.max(...k.map(r=>r.right))-q.right}})`));
  await b.eval("document.querySelectorAll('.startup-profile').forEach(c=>c.style.width='')");
  for(const r of rows){if(r.tops!==1||r.h>26||r.right>0.5||!/Edit/.test(r.labels))throw Error(width+' '+theme+' agent card tool row leaves one line: '+JSON.stringify(r));}
 }
 console.log(width,'agent card tool rows stay on one line');
}finally{await b.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
