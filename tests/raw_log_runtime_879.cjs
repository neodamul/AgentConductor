// #879: multiline raw-log display in actual task and coordinator views.
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome)throw Error('Chrome is required');const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-raw-text-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);browser=await launch(chrome,{width:1280,height:900});
for(const width of [840,1280])for(const theme of ['light','dark']){
 await browser.viewport(width,900);await browser.goto(pathToFileURL(file).href);
 const checks=await browser.eval(`(async()=>{const q=id=>document.getElementById(id),out=[],check=(n,ok)=>{if(!ok)throw Error(n);out.push(n);};const until=async f=>{const end=Date.now()+10000;while(!f()){if(Date.now()>end)throw Error('UI did not settle');await new Promise(r=>setTimeout(r,30));}};
 document.documentElement.dataset.theme='${theme}';const source=JSON.stringify({type:'assistant',message:{text:'first log line\\nsecond log line\\n<img src=x>'},usage:{input:12}})+'\\n'+JSON.stringify({type:'result',result:'done\\nnext'})+'\\n';
 SIM.reply.task=p=>({task:SIM.room.report.tasks[p.index],events:[],decisions:[],knowledge:[],step_lessons:[],usage:null,stdout:source,stderr:'error first\\nerror second'});SIM.reply.coordinator=()=>({history:[],judgments:[],stdout:source,stderr:'coordinator first\\ncoordinator second'});SIM.home.run_available=true;SIM.home.run_state='running';await until(()=>!q('start-tab-tasks').disabled);q('start-tab-tasks').click();await until(()=>q('stdout').textContent.includes('first log line'));document.querySelector('[data-tab="logs"]').click();
 const shown=q('stdout').textContent;check('task JSON message line breaks are real',shown.includes('first log line\\nsecond log line\\n<img src=x>')&&shown.includes('done\\nnext'));check('record metadata remains visible',shown.includes('"input": 12')&&shown.includes('"type": "result"'));check('plain stderr remains unchanged',q('stderr').textContent==='error first\\nerror second');check('log payload stays text',!q('stdout').querySelector('img'));
 const text=q('stdout').firstChild,range=document.createRange();const at=text.textContent.indexOf('first log');range.setStart(text,at);range.setEnd(text,at+5);const first=range.getBoundingClientRect().top;const second=text.textContent.indexOf('second log');range.setStart(text,second);range.setEnd(text,second+6);check('message lines have separate visual rows',range.getBoundingClientRect().top>first);
 check('log keeps bounded scrolling and wrapping',getComputedStyle(q('stdout')).maxHeight==='250px'&&getComputedStyle(q('stdout')).overflowY==='auto'&&getComputedStyle(q('stdout')).overflowWrap==='anywhere');
 document.querySelector('.final-step .session-choice').click();await until(()=>q('stderr').textContent==='coordinator first\\ncoordinator second');check('coordinator uses the same multiline renderer',q('stdout').textContent===shown);return out;
 })()`);console.log(theme,width,checks.length,'raw log checks passed');
}
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
