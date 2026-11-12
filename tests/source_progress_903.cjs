// #903: the document preview window shows a guide, one progress card per reference with its live log,
// the review cards after the lookup, and retries only the failed documents.
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-source-progress-903-'));let b;try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(findChrome(),{width:1280,height:900});
for(const theme of ['light','dark']){await b.goto(pathToFileURL(file).href);console.log(theme,await b.eval(`(async()=>{const q=id=>document.getElementById('start-'+id),wait=ms=>new Promise(r=>setTimeout(r,ms)),until=async(f,n)=>{for(let i=0;i<400;i++){if(f())return;await wait(25);}throw Error('timed out: '+n);},check=(n,ok)=>{if(!ok)throw Error(n);};document.documentElement.dataset.theme='${theme}';await until(()=>q('name').value,'workspace');
const t0=Date.now()-239000,log=(at,event,detail='')=>({at_ms:t0+at,event,detail});
SIM.home.config.goal='[Jira:EMM-1] [Confluence:22] [Jira:EMM-3]';
SIM.home.job={id:'j1',kind:'mcp',state:'running',started_ms:t0};
SIM.home.mcp_progress={started_ms:t0,documents:[{reference:'jira:EMM-1',stage:'summary_wait',title:'Retry policy',started_ms:t0,ended_ms:t0+112000,log:[log(0,'coordinator','claude'),log(38000,'selected','atlassian · getIssue · 1/8'),log(40000,'mcp_call','300'),log(112000,'read_done','34816')]},{reference:'confluence:22',stage:'reading',title:'Architecture',started_ms:t0+112000,log:[log(112000,'coordinator','claude'),log(153000,'selected','atlassian · getPage · 1/8'),log(155000,'mcp_call','300')]},{reference:'jira:EMM-3',stage:'waiting',log:[]}]};
await until(()=>q('mcp-records').querySelectorAll('.source-doc').length===3,'progress cards');
const cards=[...q('mcp-records').querySelectorAll('.source-doc')];
check('running card is marked and shows its log',cards[1].classList.contains('running')&&cards[1].querySelector('.source-log').textContent.includes('00:43')&&cards[1].querySelector('.source-log').textContent.includes('300'));
check('read card reports the size in its steps',cards[0].querySelectorAll('.source-step.done').length===2&&cards[0].querySelector('.source-step.current'));
check('waiting card has no time yet',cards[2].querySelector('.source-time').textContent==='—'&&!cards[2].querySelector('.source-log'));
check('status beside the Goal buttons names the running document',q('source-status').textContent.includes('confluence:22')&&q('source-status').textContent.includes('2/3')===false&&q('source-status').textContent.includes('1/3'));
check('guide is open while running',q('mcp-guide').open&&q('mcp-retry').hidden&&q('mcp-meta').textContent.length>0);
let packet=null;SIM.reply.mcp_preview=p=>{packet=p;return {accepted:true};};
SIM.home.job={id:'j1',kind:'mcp',state:'completed',started_ms:t0};
SIM.home.mcp_progress.documents[0].stage='done';SIM.home.mcp_progress.documents[1].stage='done';SIM.home.mcp_progress.documents[1].ended_ms=t0+340000;
SIM.home.mcp_progress.documents[2]={reference:'jira:EMM-3',stage:'failed',started_ms:t0+340000,ended_ms:t0+640000,error:'timed out',log:[log(340000,'coordinator','claude'),log(383000,'mcp_call','300'),log(640000,'failed')]};
const rec=(kind,id,title)=>({reference:{kind,id},server:'atlassian',tool:'get',title,url:null,version:'7',updated_at:null,sha256:'abcdef0123456789',body:'x'.repeat(5000)});
SIM.home.mcp_result={id:'p1',goal_sha256:'g',settings_sha256:'s',fetched_at:Math.floor(t0/1000),records:[rec('jira','EMM-1','Retry policy'),rec('confluence','22','Architecture')],failures:[{reference:{kind:'jira',id:'EMM-3'},error:'timed out'}],summaries:{'jira:EMM-1':{text:'Summary one'},'confluence:22':{text:'Summary two'}}};
await until(()=>q('mcp-review').open&&q('mcp-records').querySelector('.source-summary'),'review cards');
const done=[...q('mcp-records').querySelectorAll('.source-doc')];
check('review cards keep the order and carry summaries',done.length===3&&done[0].querySelector('.source-summary').textContent==='Summary one'&&done[1].querySelector('input').checked);
check('failed card shows the failed step and error',done[2].querySelector('.source-step.failed')&&done[2].querySelector('.source-step.failed').textContent.length>0&&done[2].querySelector('.source-error').textContent==='timed out'&&done[2].querySelector('input').disabled);
check('a failed Jira document blocks apply and offers retry',q('mcp-apply').disabled&&!q('mcp-retry').hidden&&!q('mcp-guide').open);
check('focus starts on Close, not the guide',document.activeElement===q('mcp-review-close'));
const head=done[0].querySelector('.source-doc-head').getBoundingClientRect();check('card head is one line',head.height<30);
q('mcp-retry').click();await until(()=>packet,'retry request');check('retry asks only for failed documents',packet.kind==='mcp_preview'&&packet.retry===true);
q('mcp-review').close();return '11 preview window checks passed';})()`));}
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
