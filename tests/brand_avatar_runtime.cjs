// #435: status boxes paint trusted embedded SVG paths directly on both agent and task lists.
const assert=require('node:assert/strict'),fs=require('node:fs'),os=require('node:os'),path=require('node:path');
const {execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{
 const chrome=findChrome();if(!chrome){console.log('brand avatar skipped: no Chrome');return;}
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-brand-avatar-'));let browser;
 try{
  const html=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),html]);
  browser=await launch(chrome);await browser.goto(pathToFileURL(html).href);
  await browser.eval(`sessionStorage.setItem('SIM_PRESET',"SIM.home.icons={claude:'data:image/svg+xml;base64,PHN2Zy8+',codex:'data:image/svg+xml;base64,PHN2Zy8+'};SIM.room.icons=SIM.home.icons;SIM.room.report.plan.tasks[0].agent='claude';");location.reload()`);
  await new Promise(r=>setTimeout(r,1200));
  const check=async selector=>{
   const atom=await browser.eval(`(()=>{const e=[...document.querySelectorAll(${JSON.stringify(selector)})].find(e=>e.getBoundingClientRect().width>0),svg=e?.matches('svg')?e:e?.querySelector('svg');const c=e&&getComputedStyle(e);return {tag:e?.tagName,svg:!!svg,path:svg?.querySelector('path')?.getAttribute('d'),width:e?.getBoundingClientRect().width,mark:e?.getBoundingClientRect().width-2*(parseFloat(c?.borderLeftWidth)+parseFloat(c?.paddingLeft)),images:e?.querySelectorAll('image,img').length};})()`);
   assert.notEqual(atom.tag,'IMG','border and SVG share the same inline drawing tree');assert.ok(atom.svg);assert.ok(atom.path?.length>100);assert.equal(atom.images,0);
   return atom;
  };
  assert.equal(await browser.eval("acBrandAvatar('custom','test')"),null);
  const agent=await check('.startup-profile .agent-symbol');assert.equal(agent.width,29);assert.equal(agent.mark,19);
  await browser.eval(`sessionStorage.setItem('SIM_PRESET',\"SIM.home.icons={claude:'data:image/svg+xml;base64,PHN2Zy8+',codex:'data:image/svg+xml;base64,PHN2Zy8+'};SIM.room.icons=SIM.home.icons;SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.plan.tasks[0].agent='claude';\");location.reload()`);
  await new Promise(r=>setTimeout(r,1200));
  await browser.eval("document.getElementById('start-tab-tasks').click()");
  await new Promise(r=>setTimeout(r,600));
  const task=await check('.outline-task .session-symbol');assert.equal(task.width,28);assert.equal(task.mark,20);
  assert.equal(task.path,agent.path,'both factories use the same pinned asset, not the snapshot URL');
  console.log('brand avatars: trusted inline paths and exact agent/task content boxes pass');
 }finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}
})().catch(e=>{console.error(e);process.exitCode=1;});
