// #878: the real task Goal starts folded and preserves visible authored line breaks.
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome)throw Error('Chrome is required');const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-goal-text-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);browser=await launch(chrome,{width:1280,height:900});
for(const width of [840,1280])for(const theme of ['light','dark']){
 await browser.viewport(width,900);await browser.goto(pathToFileURL(file).href);
 const sharedCss=fs.readFileSync('src/ui/room.css','utf8');
 const checks=await browser.eval(`(async()=>{const q=id=>document.getElementById(id),out=[],check=(n,ok)=>{if(!ok)throw Error(n);out.push(n);};const until=async f=>{const end=Date.now()+10000;while(!f()){if(Date.now()>end)throw Error('UI did not settle');await new Promise(r=>setTimeout(r,30));}};
 document.documentElement.dataset.theme='${theme}';SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.plan.goal='First Goal line\\n\\nSecond Goal line\\nThird <b>literal</b> line';await until(()=>!q('start-tab-tasks').disabled);q('start-tab-tasks').click();await until(()=>document.body.classList.contains('on-run')&&q('goal-full').textContent===SIM.room.report.plan.goal);
 const full=q('goal-full'),toggle=q('goal-toggle');check('Goal starts collapsed',full.hidden&&toggle.getAttribute('aria-expanded')==='false');toggle.click();
 check('expanded Goal preserves authored text',!full.hidden&&full.textContent===SIM.room.report.plan.goal&&!full.querySelector('b'));
 check('expanded Goal wraps and scrolls within its bound',getComputedStyle(full).whiteSpace==='pre-wrap'&&getComputedStyle(full).overflowY==='auto'&&getComputedStyle(full).maxHeight==='160px');
 const text=full.firstChild,range=document.createRange();range.setStart(text,0);range.setEnd(text,5);const first=range.getBoundingClientRect().top;const index=text.textContent.indexOf('Second');range.setStart(text,index);range.setEnd(text,index+6);check('second authored line is visibly lower',range.getBoundingClientRect().top>first+20);
 for(const id of ['start-tab-verify','start-tab-merge']){q(id).click();check(id+' keeps the same multiline Goal surface',!full.hidden&&full.textContent===SIM.room.report.plan.goal);}
 toggle.click();check('Collapse restores compact heading',full.hidden&&toggle.getAttribute('aria-expanded')==='false');toggle.click();SIM.room.report.plan.goal='New Goal first line\\nNew Goal second line';await until(()=>full.textContent===SIM.room.report.plan.goal);check('a different Goal starts collapsed',full.hidden&&toggle.getAttribute('aria-expanded')==='false');toggle.click();const style=document.querySelector('style');style.textContent=${JSON.stringify(sharedCss)};check('standalone Room uses the same pre-wrapped Goal style',getComputedStyle(full).whiteSpace==='pre-wrap'&&getComputedStyle(full).maxHeight==='160px');return out;
 })()`);console.log(theme,width,checks.length,'Goal text checks passed');
}
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
