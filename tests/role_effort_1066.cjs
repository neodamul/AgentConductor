// GitHub #1066: the Agent roles step has an Effort per role; it is saved with the role rule, and empty keeps the agent default.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('role effort skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-role-effort-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);await wait(1500);
 const until=async(code,name)=>{for(let i=0;i<150;i++){if(await b.eval(code).catch(()=>false))return;await wait(100);}fail('timed out: '+name);};
 await b.eval("document.getElementById('start-tab-team').click()");
 await until("document.getElementById('start-role-development-effort')",'effort column');
 const head=await b.eval("[...document.querySelectorAll('#start-roles thead th')].map(t=>t.textContent.trim()).join('|')");
 if(!/Effort/.test(head))fail('no Effort header: '+head);
 const opts=await b.eval("[...document.getElementById('start-role-development-effort').options].map(o=>o.value).join(',')");
 if(opts!==',low,medium,high,xhigh,max')fail('unexpected effort options: '+opts);
 await b.eval("{const s=document.getElementById('start-role-development-agent');if(!s.value){s.value=s.options[1].value;s.dispatchEvent(new Event('change',{bubbles:true}));}const e=document.getElementById('start-role-development-effort');e.value='high';e.dispatchEvent(new Event('change',{bubbles:true}));}");
 const rules=await b.eval("JSON.stringify(acRoleChoices.read(document.getElementById('start-roles')))");
 const parsed=JSON.parse(rules);
 if(parsed.development?.[0]?.effort!=='high')fail('the role effort is not read: '+rules);
 if(Object.entries(parsed).some(([role,[e]])=>role!=='development'&&'effort' in e))fail('an empty effort was stored: '+rules);
 // Fallback Effort follows the fallback agent: locked without one, saved inside the fallback policy with one.
 await b.eval("{const f=document.getElementById('start-role-development-fallback');f.value='';f.dispatchEvent(new Event('change',{bubbles:true}));}");
 if(!await b.eval("document.getElementById('start-role-development-fallback-effort').disabled"))fail('fallback effort is open without a fallback agent');
 await b.eval("{const f=document.getElementById('start-role-development-fallback');const a=document.getElementById('start-role-development-agent').value;f.value=[...f.options].find(o=>o.value&&o.value!==a&&!o.value.startsWith('__')&&!o.disabled).value;f.dispatchEvent(new Event('change',{bubbles:true}));const fe=document.getElementById('start-role-development-fallback-effort');fe.value='medium';fe.dispatchEvent(new Event('change',{bubbles:true}));}");
 const withFallback=JSON.parse(await b.eval("JSON.stringify(acRoleChoices.read(document.getElementById('start-roles')))"));
 if(withFallback.development[0].fallback?.effort!=='medium')fail('the fallback effort is not saved: '+JSON.stringify(withFallback.development));
 console.log('role effort: the column offers the agent default and five levels, a chosen effort is saved with its role, and Fallback Effort follows the fallback agent');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
