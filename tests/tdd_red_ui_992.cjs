// #992: every verdict honors the native result, including a nonzero Red pass and a zero-exit unexpected pass.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),vm=require('node:vm'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {launch,findChrome}=require('./design_parity/cdp.cjs');
const ctx=vm.createContext({window:{}});
vm.runInContext(fs.readFileSync('src/ui/result_view.js','utf8')+fs.readFileSync('src/ui/room_verify.js','utf8'),ctx);
assert.equal(vm.runInContext("window.acResultView.tests([{passed:true,exit_code:1},{passed:false,exit_code:0},{exit_code:0}]).passed",ctx),2);
assert.equal(vm.runInContext("isFailedVerification({state:'failed',tasks:[{status:'completed'}],tests:[{passed:true,exit_code:1}]})",ctx),false);
assert.equal(vm.runInContext("isFailedVerification({state:'failed',tasks:[{status:'completed'}],tests:[{passed:false,exit_code:0}]})",ctx),true);
(async()=>{let b;const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-red-992-'));
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(findChrome(),{width:1280,height:900});
for(const theme of ['light','dark']){
 await b.goto(pathToFileURL(file).href);
 await b.eval(`document.documentElement.dataset.theme='${theme}';SIM.home.run_available=true;SIM.home.run_state='running';const r=SIM.room.report;r.state='waiting_approval';r.tasks=r.tasks.slice(0,1);r.plan.tasks=r.plan.tasks.slice(0,1);r.tasks[0].status='completed';r.plan.tasks[0].test_expect_failure=true;r.candidate={commit:'a'.repeat(40)};r.tests=[{task:r.tasks[0].id,exit_code:1,timed_out:false,passed:true}];SIM.reply.evidence=()=>({report:r,test_output:r.tests,diff:'',truncated:false});`);
 const until=async(expr)=>{for(let i=0;i<150;i++){if(await b.eval(expr))return;await new Promise(r=>setTimeout(r,50));}throw Error('UI did not settle: '+expr);};
 await until("!document.getElementById('start-tab-tasks').disabled");
 await b.eval("document.getElementById('start-tab-tasks').click()");
 await until("!!document.querySelector('#review-summary .ok')");
 assert.equal(await b.eval("document.getElementById('run-error').hidden"),true);
 await b.eval("SIM.room.report.state='failed';SIM.room.report.tests[0].passed=false;SIM.room.report.tests[0].exit_code=0;SIM.room.report.error='integration test failed: expected failure';SIM.room.revision='unexpected';");
 await until("!!document.querySelector('.verify-fail')&&!document.getElementById('run-error').hidden");

 await b.goto(pathToFileURL(file).href);
 await b.eval("SIM.home.proposal=SIM.proposal('ready');SIM.home.proposal.plan.tasks[0].test_expect_failure=true;");
 await until("!!document.querySelector('#start-tasks [data-key=prompt]')");
 assert.equal(await b.eval("document.getElementById('start-tasks').textContent.includes(acMessage('startup.expected_test_failure'))"),true,'Red expectation is visible to the reviewer');
 await b.eval("document.getElementById('start-tab-review').click();const p=document.querySelector('#start-tasks [data-key=prompt]');p.value+=' reviewed';p.dispatchEvent(new Event('input',{bubbles:true}));document.getElementById('start-edit').click();");
 await until("SIM.log.some(e=>e.kind==='edit')");
 assert.equal(await b.eval("SIM.log.find(e=>e.kind==='edit').p.tasks[0].test_expect_failure"),true,'plan editing preserves the expected-failure marker');
 console.log(theme,'Red pass renders as success; unexpected pass exposes failure and recovery');
}
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
