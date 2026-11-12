// #907: a Goal holds at most five distinct references; adding stops at five and a sixth blocks preview.
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-reference-limit-907-'));let b;try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(findChrome(),{width:1280,height:900});
for(const theme of ['light','dark']){await b.goto(pathToFileURL(file).href);console.log(theme,await b.eval(`(async()=>{const q=id=>document.getElementById('start-'+id),wait=ms=>new Promise(r=>setTimeout(r,ms)),until=async(f,n)=>{for(let i=0;i<200;i++){if(f())return;await wait(25);}throw Error('timed out: '+n);},check=(n,ok)=>{if(!ok)throw Error(n);};document.documentElement.dataset.theme='${theme}';await until(()=>q('name').value,'workspace');
SIM.reply.mcp_source_suggestions=p=>{const refs=[...new Set([...p.goal.matchAll(/\\[(Jira|Confluence):([^\\]]+)\\]/g)].map(m=>m[1].toLowerCase()+':'+m[2]))];return {references:refs,reference_ranges:[],suggestions:[]};};
const set=async(v,n)=>{q('goal').value=v;q('goal').dispatchEvent(new Event('input',{bubbles:true}));await until(()=>q('source-status').textContent.includes(n),'status '+n);};
await set('[Jira:A-1] [Jira:A-2] [Confluence:3] [Confluence:4]','4/5');check('four references allow adding',!q('jira-reference').disabled&&!q('mcp-preview').disabled);
await set('[Jira:A-1] [Jira:A-2] [Confluence:3] [Confluence:4] [Jira:A-1] [Jira:A-5]','5/5');check('five distinct references stop adding but still preview',q('jira-reference').disabled&&q('confluence-reference').disabled&&q('jira-reference').title.length>0&&!q('mcp-preview').disabled);
await set('[Jira:A-1] [Jira:A-2] [Confluence:3] [Confluence:4] [Jira:A-5] [Jira:A-6]','(');check('a sixth reference blocks preview with an inline error',q('source-status').classList.contains('over')&&q('mcp-preview').disabled&&getComputedStyle(q('source-status')).color!==getComputedStyle(q('goal')).color);
await set('Build the thing','');await until(()=>!q('jira-reference').disabled,'restored');check('removing references restores the controls',!q('source-status').classList.contains('over'));
return '4 reference limit checks passed';})()`));}
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
