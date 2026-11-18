// GitHub #1071: a custom CLI profile can carry a display name; it is saved with the profile and shown instead of the id.
const fs=require('fs'),path=require('path'),os=require('os'),vm=require('vm'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const root=path.resolve(__dirname,'..');
// acAgentName: profile name, then the provider name, then the id; duplicates keep the "Name · id" suffix.
{const src=fs.readFileSync(path.join(root,'src/ui/i18n.js'),'utf8'),start=src.indexOf('const AC_AGENT_NAMES');const ctx={};vm.runInNewContext(src.slice(start)+';this.f=acAgentName;',ctx);const f=ctx.f;
 const eq=(a,b,m)=>{if(a!==b)throw Error(m+': '+a);};
 eq(f({id:'my-cli',parser:'custom'}),'my-cli','unnamed custom shows the id');
 eq(f({id:'my-cli',parser:'custom',name:'My CLI'}),'My CLI','named custom shows its name');
 eq(f({id:'claude',parser:'claude'}),'Claude Code','built-in keeps its name');
 eq(f({id:'work',parser:'claude',name:'  '}),'Claude Code','a blank name falls back');
 const two=[{id:'a',parser:'custom',name:'Same'},{id:'b',parser:'custom',name:'Same'}];eq(f(two[0],two),'Same · a','duplicate names get the id suffix');}
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('agent display name: dialog skipped, no Chrome');process.exit(0);}
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-display-name-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);await wait(1500);
 const until=async(code,name)=>{for(let i=0;i<150;i++){if(await b.eval(code).catch(()=>false))return;await wait(100);}fail('timed out: '+name);};
 await until("document.querySelectorAll('#start-profiles .startup-profile').length===2",'agent cards');
 await b.eval("document.getElementById('start-add').click()");await until("document.getElementById('start-agent-dialog').open",'registration dialog');
 if(await b.eval("document.getElementById('start-provider').value")!=='custom')fail('a new agent is not a custom CLI');
 if(await b.eval("document.getElementById('start-agent-name-label').hidden"))fail('the display name field is hidden for a custom CLI');
 await b.eval("{const p=document.getElementById('start-provider');p.value='codex';p.onchange();}");await wait(200);
 if(!await b.eval("document.getElementById('start-agent-name-label').hidden"))fail('the display name field shows for a built-in CLI');
 await b.eval("{const p=document.getElementById('start-provider');p.value='custom';p.onchange();}");await wait(200);
 await b.eval("SIM.log.length=0;document.getElementById('start-agent-id').value='my-cli';document.getElementById('start-agent-name').value=' My CLI ';document.getElementById('start-agent-apply').click()");
 await until("SIM.log.some(x=>x.kind==='save_profiles')",'profiles saved');
 const saved=await b.eval("JSON.stringify(SIM.log.find(x=>x.kind==='save_profiles').p.profiles.map(p=>[p.id,p.name||null]))");
 if(saved!==JSON.stringify([['claude',null],['codex',null],['my-cli','My CLI']]))fail('unexpected saved names: '+saved);
 console.log('agent display name: custom profiles save a trimmed display name, built-ins have none, and acAgentName prefers it');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
