// GitHub #1082: plan review edits make Save plan edits primary, a model outside the catalog stays shown,
// and after saving, the rail outline of a phased plan shows the saved agent.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('plan edit save skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-plan-edit-1082-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
const preset=`SIM.home.proposal=SIM.proposal('ready');SIM.home.proposal.plan.tasks[0].model='custom-model-x';SIM.home.slugs=[{slug:'g',goal:'g',active:true,plan:{phases:[{id:'p1',brief:'',specs:[]}]},
 progress:{stage:'plan',current_phase:'p1',phases:[{id:'p1',name:'Build',stage:'plan',task:[],completed:[],verification:[]}]}}];SIM.home.proposal.slug='g';SIM.home.proposal.slug_phase='p1';
 for(const list of [SIM.home.config.profiles,SIM.home.proposal.plan.agents]){const c=list.find(x=>x.id==='codex');c.parser='custom';c.name='Zeta';}
 SIM.reply.edit=p=>{SIM.home.proposal.plan.tasks=p.tasks;SIM.home.proposal.digest='edited-digest';SIM.home.revision='r2';return {};};`;
const until=async(b,expr,what)=>{for(let i=0;i<100;i++){if(await b.eval(expr).catch(()=>false))return;await wait(100);}fail(what);};
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 const url=pathToFileURL(path.join(dir,'app.html')).href;await b.goto(url);await b.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(preset)})`);await b.goto(url);
 await until(b,"!!document.querySelector('#start-tasks [data-key=agent]')",'plan did not render');
 await b.eval("document.getElementById('start-tab-review').click()");await wait(300);
 const model=await b.eval("document.querySelector('#start-tasks [data-key=model]').value");
 if(model!=='custom-model-x')fail('a model outside the catalog must stay shown, got '+JSON.stringify(model));
 const idle=await b.eval("(()=>{const e=document.getElementById('start-edit');return {disabled:e.disabled,bg:getComputedStyle(e).backgroundColor}})()");
 if(!idle.disabled)fail('Save plan edits starts disabled');
 await b.eval("(()=>{const m=document.querySelector('#start-tasks [data-key=model]');m.value='';m.dispatchEvent(new Event('input',{bubbles:true}));})()");
 if(await b.eval("document.getElementById('start-edit').disabled"))fail('a model edit must enable Save plan edits');
 await b.eval("(()=>{const m=document.querySelector('#start-tasks [data-key=model]');m.value='custom-model-x';m.dispatchEvent(new Event('input',{bubbles:true}));})()");
 await b.eval("(()=>{const a=document.querySelector('#start-tasks [data-key=agent]');a.value=[...a.options].find(o=>o.value!==a.value).value;a.dispatchEvent(new Event('input',{bubbles:true}));a.dispatchEvent(new Event('change',{bubbles:true}));})()");
 const dirty=await b.eval("(()=>{const e=document.getElementById('start-edit'),r=document.getElementById('start-run');return {disabled:e.disabled,bg:getComputedStyle(e).backgroundColor,primary:getComputedStyle(r).backgroundColor,agent:document.querySelector('#start-tasks [data-key=agent]').value}})()");
 if(dirty.disabled)fail('an agent edit must enable Save plan edits');
 if(dirty.bg===idle.bg)fail('enabled Save plan edits must change to the primary style: '+JSON.stringify([idle,dirty]));
 await b.eval("document.getElementById('start-edit').click()");
 await until(b,"document.getElementById('start-edit').disabled",'save did not finish');
 await until(b,`[...document.querySelectorAll('#rail-outline-body .outline-task')].some(r=>r.querySelector('.outline-agent')?.textContent===${JSON.stringify(dirty.agent==='claude'?'C':'Z')}&&/T1/.test(r.textContent))`,'the rail outline must show the saved agent after Save plan edits');
 console.log('plan edits: primary save button, off-catalog model kept, outline follows the saved agent');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
