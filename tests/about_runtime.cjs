// #468: the About dialog opens centered on the first start after an install or update, records the version when it is
// closed, and is reachable from the menu; copy buttons put the install command on the clipboard.
const assert=require('assert/strict'),fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('about skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-about-'));
const about=fs.readFileSync(path.join(root,'src/ui/about.html'),'utf8').match(/class="about-mark" src="data:image\/png;base64,([^"]+)"/);
assert.ok(about,'About embeds its application mark');
assert.deepEqual(Buffer.from(about[1],'base64'),fs.readFileSync(path.join(root,'src/ui/assets/ac-64.png')),'About uses the current application icon asset');
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const cases={first:"SIM.reply.about_status=()=>({version:'0.2.0',show:true,updated:false});",update:"SIM.reply.about_status=()=>({version:'0.2.0',show:true,updated:true});",quiet:"SIM.reply.about_status=()=>({version:'0.2.0',show:false,updated:false});"};
(async()=>{try{for(const width of [840,1280])for(const theme of ['light','dark']){const browser=await launch(chrome,{width,height:960});try{
 const result={};
 for(const [name,preset] of Object.entries(cases)){
  await browser.goto(pathToFileURL(path.join(dir,'app.html')).href+'?run='+name+theme+width);
  await browser.eval("window.__beforePreset=true");
  await browser.eval(`PRESET(${JSON.stringify(preset+'window.aboutFixture=true;')})`);
  for(let i=0;i<300;i++){let ok=false;try{ok=await browser.eval("!window.__beforePreset && window.aboutFixture===true");}catch{}if(ok)break;await new Promise(r=>setTimeout(r,50));}
  await new Promise(r=>setTimeout(r,500));
  result[name]=await browser.eval(`(async()=>{
   const checks=[],check=(n,v)=>{if(!v)throw Error(n);checks.push(n);};const wait=ms=>new Promise(r=>setTimeout(r,ms));
   document.documentElement.dataset.theme=${JSON.stringify(theme)};
   const d=document.getElementById('about-dialog'),tag=document.getElementById('about-tag');
   const seen=()=>SIM.log.filter(e=>e.kind==='about_seen').length;
   if(${JSON.stringify(name)}==='quiet'){check('already shown version stays closed',!d.open);}
   else{
    for(let i=0;i<200&&!d.open;i++)await wait(50);
    check('opens by itself',d.open);
    const r=d.getBoundingClientRect(),cx=r.left+r.width/2,cy=r.top+r.height/2;
    check('centered in the window',Math.abs(cx-innerWidth/2)<2&&Math.abs(cy-innerHeight/2)<2);
    check('update tag follows the update flag',tag.hidden===(${JSON.stringify(name)}!=='update')&&(${JSON.stringify(name)}!=='update'||tag.textContent.includes('0.2.0')));
    check('install commands listed',[...d.querySelectorAll('code')].map(c=>c.textContent).join('|').includes('npm install -g @anthropic-ai/claude-code')&&d.textContent.includes('@openai/codex')&&d.textContent.includes('opencode-ai'));
    check('version shown',document.getElementById('about-meta').textContent.includes('0.2.0'));
    check('start button focused',document.activeElement.id==='about-start');
    check('nothing recorded before closing',seen()===0);
    d.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));document.getElementById('about-start').click();for(let i=0;i<100&&(d.open||seen()<1);i++)await wait(50);await wait(100);
    check('closing records the version',!d.open&&seen()===1);
   }
   document.getElementById('start-app-menu').click();await wait(300);
   const item=[...document.querySelectorAll('#start-workspace-list button')].find(b=>b.textContent.trim()===acMessage('about.menu'));
   check('menu entry exists',!!item);item.click();await wait(400);
   check('menu opens it without the update tag',d.open&&tag.hidden);
   document.getElementById('about-close').click();for(let i=0;i<100&&d.open;i++)await wait(50);check('close button closes it',!d.open);
   return checks.length;
  })()`);
 }
 console.log(theme,width,JSON.stringify(result),'about checks passed');
}finally{await browser.close();}}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
