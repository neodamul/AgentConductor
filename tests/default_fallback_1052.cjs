// GitHub #1052: a newly registered agent falls back to a registered one (Claude Code first), and that agent falls back
// to it when it had none, so both cards show the Alternate tag as the design does.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('default fallback skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-default-fallback-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);await wait(1500);
 const until=async(code,name)=>{for(let i=0;i<150;i++){if(await b.eval(code).catch(()=>false))return;await wait(100);}fail('timed out: '+name);};
 await until("document.querySelectorAll('#start-profiles .startup-profile').length===2",'two agent cards');
 if(await b.eval("SIM.home.config.profiles.some(p=>p.fallback)"))fail('fixture agents already have fallbacks');
 await b.eval("document.getElementById('start-add').click()");await until("document.getElementById('start-agent-dialog').open",'registration dialog');
 const chosen=await b.eval("document.getElementById('start-fallback-agent').value");if(chosen!=='claude')fail('the new agent does not fall back to Claude Code: '+chosen);
 await b.eval("SIM.log.length=0;document.getElementById('start-agent-id').value='third';document.getElementById('start-agent-apply').click()");
 await until("SIM.log.some(x=>x.kind==='save_profiles')",'profiles saved');
 const saved=await b.eval("JSON.stringify(SIM.log.find(x=>x.kind==='save_profiles').p.profiles.map(p=>[p.id,p.fallback&&p.fallback.agent]))");
 if(saved!==JSON.stringify([['claude','third'],['codex',null],['third','claude']]))fail('unexpected fallbacks: '+saved);
 // An agent that already has a fallback keeps it.
 await b.eval("SIM.home.config.profiles[0].fallback={agent:'codex',model:null}");await wait(2500);
 await b.eval("document.getElementById('start-add').click()");await until("document.getElementById('start-agent-dialog').open",'second registration');
 await b.eval("SIM.log.length=0;document.getElementById('start-agent-id').value='fourth';document.getElementById('start-agent-apply').click()");
 await until("SIM.log.some(x=>x.kind==='save_profiles')",'second save');
 const kept=await b.eval("JSON.stringify(SIM.log.find(x=>x.kind==='save_profiles').p.profiles.map(p=>[p.id,p.fallback&&p.fallback.agent]))");
 if(!kept.includes('["claude","codex"]')||!kept.includes('["fourth","claude"]'))fail('an existing fallback was replaced: '+kept);
 console.log('default fallback: a new agent and its target fall back to each other; existing fallbacks are kept');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
