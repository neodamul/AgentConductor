// GitHub #869/#870: the coordinator card leads with a role tag and band, every tag has a thin border and 4px corners,
// and the models tag lists the queried models on hover or keyboard focus (at most 8, then a remainder line).
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('agent tags skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-agent-tags-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
(async()=>{try{const browser=await launch(chrome,{width:1280,height:900});try{
 await browser.goto(pathToFileURL(path.join(dir,'app.html')).href);
 const result=await browser.eval(`(async()=>{
  const wait=async(f,why)=>{const end=Date.now()+8000;while(!f()&&Date.now()<end)await new Promise(r=>setTimeout(r,30));if(!f())throw Error(why);};
  await wait(()=>document.querySelectorAll('.startup-profile').length>=2,'agent cards');
  const coord=document.querySelector('.startup-profile.coordinator');if(!coord)throw Error('the coordinator card is marked');
  const role=coord.querySelector('.work-facts').firstElementChild;if(!role.classList.contains('role'))throw Error('the role tag leads the first row');
  if(document.querySelectorAll('.fact.role').length!==1)throw Error('only the coordinator has a role tag');
  if(!getComputedStyle(coord).boxShadow.includes('inset'))throw Error('the coordinator card has a role band');
  const tags=[...document.querySelectorAll('.startup-profile .agent-facts .fact')];
  if(!tags.every(t=>{const c=getComputedStyle(t);return c.borderTopWidth==='1px'&&c.borderTopLeftRadius==='4px';}))throw Error('thin borders and 4px corners');
  const p=SIM.home.config.profiles.find(x=>x.id==='claude');p.catalog.models=Array.from({length:11},(_,i)=>'model-'+i);SIM.home.revision=(SIM.home.revision||0)+1;
  await wait(()=>[...document.querySelectorAll('.startup-profile[data-agent="claude"] .model-list span')].length===9,'the list caps at 8 and adds a remainder line');
  const tag=document.querySelector('.startup-profile[data-agent="claude"] .fact.models'),list=tag.querySelector('.model-list');
  if(getComputedStyle(list).display!=='none')throw Error('hidden until hovered or focused');
  tag.focus();tag.dispatchEvent(new FocusEvent('focusin',{bubbles:true}));
  if(getComputedStyle(list).display!=='block'||list.querySelector('.more').textContent!==acMessage('startup.models_more',{value1:3}))throw Error('focus opens the list');
  if(tag.getAttribute('aria-describedby')!==list.id)throw Error('the list describes the tag');
  tag.blur();tag.dispatchEvent(new FocusEvent('focusout',{bubbles:true}));if(getComputedStyle(list).display!=='none')throw Error('leaving closes it');
  return 'ok';
 })()`);
 console.log('agent tags',result);
}finally{await browser.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
