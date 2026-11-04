// #424: check the assembled agent-card controls, not a copied CSS fixture.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
const chrome=findChrome();if(!chrome){console.log('agent buttons skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-agent-buttons-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
(async()=>{try{for(const width of [840,1280]){const browser=await launch(chrome,{width,height:960});try{for(const theme of ['light','dark']){
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href);
 const result=await browser.eval(`(() => {
 document.documentElement.dataset.theme='${theme}';const buttons=[...document.querySelectorAll('.profile-tools button')];
 if(!buttons.length)throw Error('agent controls missing');
 for(const t of document.querySelectorAll('.profile-tools')){t.style.width='187px';for(const b of t.children)b.style.letterSpacing='2px';if(new Set([...t.children].map(b=>b.getBoundingClientRect().y)).size!==1)throw Error('actions wrap in the reported 187px strip');}
 const checks=[];for(const b of buttons){const r=b.getBoundingClientRect(),s=getComputedStyle(b),icon=b.querySelector('svg');
  if(r.height!==22)throw Error('agent button height '+r.height+' for '+b.getAttribute('aria-label'));
  if(!b.getAttribute('aria-label')||!b.title)throw Error('missing accessible button name');
  const onlyIcon=!!icon&&!b.textContent.trim()||b.classList.contains('move-agent');
  if(!onlyIcon&&(parseFloat(s.paddingLeft)>4||parseFloat(s.paddingRight)>4))throw Error('labeled action padding is excessive');
  if(onlyIcon&&(parseFloat(s.paddingLeft)!==0||parseFloat(s.paddingRight)!==0))throw Error('icon-only button still has horizontal padding');
  if(b.scrollWidth>b.clientWidth)throw Error('agent action content overflows its target');
  if(icon){const i=icon.getBoundingClientRect();if(i.width!==13||i.height!==13||i.left<r.left||i.right>r.right)throw Error('icon compressed or outside its button');}
  if(!b.disabled&&s.cursor!=='pointer')throw Error('button pointer cursor missing');
  checks.push(b.getAttribute('aria-label'));
 }return checks.length;
 })()`);console.log(theme,width,result,'agent buttons checked');
}}finally{await browser.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
