// #424 / #435: preserve icon fit and compare the current approved tools, without restoring the superseded grid.
const assert=require('node:assert/strict'),fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
const chrome=findChrome();if(!chrome){console.log('agent buttons skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-agent-buttons-'));
execFileSync(process.execPath,[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
(async()=>{try{for(const width of [840,1280]){const browser=await launch(chrome,{width,height:960});try{for(const theme of ['light','dark']){
 await browser.goto(pathToFileURL(path.join(root,'docs/design/approved-goal-ui/preview.html')).href+'?theme='+theme+'#0');
 const expected=await browser.eval(`(()=>{const e=document.querySelector('.agent .tools');return {width:e.getBoundingClientRect().width,height:e.getBoundingClientRect().height,buttons:[...e.children].map(b=>{const r=b.getBoundingClientRect();return {width:r.width,height:r.height,top:r.top-e.getBoundingClientRect().top};})};})()`);
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href);await browser.eval(`document.documentElement.dataset.theme='${theme}'`);
 const actual=await browser.eval(`(()=>{const rows=[...document.querySelectorAll('.profile-tools')];if(!rows.length)throw Error('agent controls missing');return rows.map(row=>{
  const buttons=[...row.querySelectorAll('button')];if(buttons.length!==5)throw Error('missing action');
  for(const b of buttons){const r=b.getBoundingClientRect(),s=getComputedStyle(b),icon=b.querySelector('svg');
   if(!b.getAttribute('aria-label')||!b.title)throw Error('missing accessible button name');
   if(b.scrollWidth>b.clientWidth)throw Error('agent action content overflows its target');
   if(icon){const i=icon.getBoundingClientRect();if(i.width!==13||i.height!==13||i.left<r.left||i.right>r.right)throw Error('icon compressed or outside button');}
   if(!b.disabled){if(s.cursor!=='pointer')throw Error('pointer cursor missing');b.focus();if(document.activeElement!==b)throw Error('button cannot receive focus');}
  }
  const r=row.getBoundingClientRect();return {width:r.width,height:r.height,buttons:buttons.map(b=>{const q=b.getBoundingClientRect();return {width:q.width,height:q.height,top:q.top-r.top};})};
 });})()`);
 for(const row of actual)assert.deepEqual(row,expected,'tools match the approved natural layout; narrow single-row acceptance remains #424');
 console.log(theme,width,actual.length*5,'agent buttons checked against approved layout');
}}finally{await browser.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
