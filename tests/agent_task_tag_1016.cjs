// GitHub #1016: an agent card shows how many tasks the current plan assigns to it, and hover or
// keyboard focus lists them by number and first prompt line; an agent with none shows no tag.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('agent task tag skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-agent-task-tag-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
(async()=>{try{const browser=await launch(chrome,{width:1280,height:900});try{
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href);
 const result=await browser.eval(`(async()=>{
  const wait=async(f,why)=>{const end=Date.now()+8000;while(!f()&&Date.now()<end)await new Promise(r=>setTimeout(r,30));if(!f())throw Error(why);};
  await wait(()=>document.querySelectorAll('.startup-profile').length>=2,'agent cards');
  if(document.querySelector('.startup-profile .fact.tasks'))throw Error('no plan, no task tag');
  SIM.home.proposal=SIM.proposal('ready');SIM.home.revision='r-1016';
  const tag=id=>document.querySelector('.startup-profile[data-agent="'+id+'"] .fact.tasks');
  await wait(()=>tag('codex')&&tag('claude'),'both agents get a task tag');
  const codex=tag('codex'),list=codex.querySelector('.model-list');
  if(!codex.closest('.work-facts'))throw Error('the tag sits on the work row');
  if(!codex.firstChild.textContent.includes(acMessage('startup.assigned_tasks',{value1:2})))throw Error('codex has 2 tasks: '+codex.textContent);
  const rows=[...list.querySelectorAll('span')].map(s=>s.textContent);
  if(JSON.stringify(rows)!==JSON.stringify(['T1 Write the parser','T3 Write the tests']))throw Error('rows '+rows);
  if(getComputedStyle(list).display!=='none')throw Error('hidden until hovered or focused');
  codex.focus();codex.dispatchEvent(new FocusEvent('focusin',{bubbles:true}));
  if(getComputedStyle(list).display!=='block'||codex.getAttribute('aria-describedby')!==list.id)throw Error('focus opens the list');
  return 'ok';
 })()`);
 console.log('agent task tag',result);
}finally{await browser.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
