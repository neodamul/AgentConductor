// #1011: the task inspector folds a long instruction to four lines, keeps its line breaks, and toggles Expand/Collapse.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {launch,findChrome}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome){console.log('Instruction fold skipped: no Chrome');return;}let b;const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-1011-'));
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(chrome,{width:1280,height:900});
const long=Array.from({length:12},(_,i)=>`${i+1}. Requirement ${i+1}: unique numbers, a bonus number and a draw history.`).join('\n');
for(const theme of ['light','dark']){
 await b.goto(pathToFileURL(file).href);
 await b.eval(`{document.documentElement.dataset.theme='${theme}';SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.plan.tasks[0].prompt=${JSON.stringify(long)};SIM.room.report.plan.tasks[1].prompt='Write the docs';}`);
 const until=async(expr)=>{for(let i=0;i<150;i++){if(await b.eval(expr))return;await new Promise(r=>setTimeout(r,50));}throw Error('UI did not settle: '+expr);};
 await until("!document.getElementById('start-tab-tasks').disabled");
 await b.eval("document.getElementById('start-tab-tasks').click()");await until("document.body.classList.contains('on-run')");
 await b.eval("document.getElementById('start-inspector-toggle').click()");await new Promise(r=>setTimeout(r,1500));
 const q=id=>`document.getElementById('${id}')`;
 assert.equal(await b.eval(`${q('task-instruction')}.classList.contains('folded')`),true,'long instruction starts folded');
 assert.equal(await b.eval(`${q('task-instruction-toggle')}.hidden`),false,'toggle shown for long text');
 assert.equal(await b.eval(`getComputedStyle(${q('task-instruction')}).whiteSpace`),'pre-wrap','line breaks kept');
 const folded=await b.eval(`${q('task-instruction')}.getBoundingClientRect().height`);
 const toggleTop=()=>b.eval(`${q('task-instruction-toggle')}.getBoundingClientRect().top`),before=await toggleTop();
 await b.eval(`${q('task-instruction-toggle')}.click()`);await new Promise(r=>setTimeout(r,200));
 const open=await b.eval(`${q('task-instruction')}.getBoundingClientRect().height`);
 assert.ok(open>folded*2,`expanded height ${open} vs folded ${folded}`);
 assert.equal(await b.eval(`${q('task-instruction-toggle')}.getAttribute('aria-expanded')`),'true');
 assert.equal(await toggleTop(),before,'GitHub #1022: Collapse stays where Expand was');
 await b.eval(`${q('task-instruction-toggle')}.click()`);await new Promise(r=>setTimeout(r,200));
 assert.equal(await b.eval(`${q('task-instruction')}.classList.contains('folded')`),true,'collapse folds again');
 // A short instruction has nothing to expand.
 await b.eval("document.querySelectorAll('.outline-task, .phase-task, [data-task-index]')[1]?.click()");
 await b.eval(`{const r=SIM.room.report;r.plan.tasks[0].prompt='Short';SIM.room.revision='short';}`);await new Promise(r=>setTimeout(r,1500));
 assert.equal(await b.eval(`${q('task-instruction-toggle')}.hidden`),true,'no toggle for short text');
 console.log(theme,'instruction folds, expands and collapses');
}
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
