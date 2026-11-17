// GitHub #684: a pending tool request is decided in the task card's footer (no modal): the notice shows the input, the footer replaces
// the direction field, Allow once / Deny send the exact request, and the footer goes away once the request is settled.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('tool decision skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-tool-decision-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);await wait(1500);
 const until=async(code,name)=>{for(let i=0;i<150;i++){if(await b.eval(code).catch(()=>false))return;await wait(100);}fail('timed out: '+name);};
 const request=id=>`{id:'${id}',status:'pending',created_at:Math.floor(Date.now()/1000),context:{task:'task-1'},request:{tool:'Bash',input:{command:'npm install jsonwebtoken@9'}}}`;
 await b.eval(`SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.tool_requests=[${request('tr1')}];`);
 await until("!document.getElementById('start-tab-tasks').disabled",'run steps available');
 await b.eval("document.getElementById('start-tab-tasks').click()");
 await until("!document.getElementById('tool-decision').hidden",'footer shown for the pending request');
 const state=await b.eval("({dialog:!!document.querySelector('dialog[open]'),direction:document.querySelector('#run-screen .composer').hidden,notice:document.getElementById('task-notice').textContent,raw:!!document.querySelector('#tool-requests details'),access:/network access|네트워크 접근/.test(document.getElementById('task-notice').textContent)})");
 if(state.dialog)fail('a modal opened for the tool request');if(!state.direction)fail('the direction field is still shown');
 if(!/npm install jsonwebtoken@9/.test(state.notice))fail('the notice does not show the command: '+state.notice);if(!state.raw)fail('the exact input is not reachable in the inspector tool requests');if(!state.access)fail('the notice does not name the network access');
 await b.eval("document.getElementById('tool-decision-allow').click()");
 await until("SIM.log.some(x=>x.kind==='action'&&x.p.action.kind==='tool_approve')",'approve sent');
 const sent=await b.eval("JSON.stringify(SIM.log.filter(x=>x.kind==='action').map(x=>x.p.action))");if(!/"sha":"tr1"/.test(sent))fail('approve did not carry the request id: '+sent);
 // settled: the footer gives the direction field back; a second request is rejected through the same footer
 await b.eval(`SIM.room.tool_requests=[]`);await until("document.getElementById('tool-decision').hidden&&!document.querySelector('#run-screen .composer').hidden",'footer gone after the request settled');
 await b.eval(`SIM.room.tool_requests=[${request('tr2')}]`);await until("!document.getElementById('tool-decision').hidden",'second request shown');
 await b.eval("document.getElementById('tool-decision-deny').click()");
 await until("SIM.log.some(x=>x.kind==='action'&&x.p.action.kind==='tool_reject'&&x.p.action.sha==='tr2')",'reject sent');
 // #1046: a slow decision (the tool check re-runs Git) is waited for as long as the tool check timeout allows, with both buttons
 // locked; once that wait also runs out, the sent decision keeps them locked, with the progress bar, until its request is settled.
 await b.eval(`SIM.room.tool_requests=[${request('tr3')}];SIM.room.verifier_timeout=5;SIM.hang.action=true;`);await until("!document.getElementById('tool-decision').hidden&&!document.getElementById('tool-decision-allow').disabled",'third request shown');
 await b.eval("document.getElementById('tool-decision-allow').click()");
 await until("SIM.log.some(x=>x.kind==='action'&&x.p.action.sha==='tr3')",'slow approve sent');
 const locked="document.getElementById('tool-decision-allow').disabled&&document.getElementById('tool-decision-deny').disabled";
 await wait(16500);
 const mid=await b.eval(`({locked:${locked},timeout:SIM.log.some(x=>x.kind==='client_timeout')})`);
 if(mid.timeout)fail('the decision timed out after 15 s although the tool check may take 20 s');if(!mid.locked)fail('the buttons are not locked while the decision runs');
 await until("SIM.log.some(x=>x.kind==='client_timeout'&&x.p.of==='action')",'decision wait runs out after the tool check timeout');await wait(300);
 if(!await b.eval(locked))fail('the buttons unlocked after the timeout although the decision was sent');
 if(!await b.eval("document.getElementById('tool-decision-allow').classList.contains('is-busy')"))fail('the progress bar stopped although the sent decision is still being applied');
 await b.eval("document.getElementById('tool-decision-allow').disabled=false;document.getElementById('tool-decision-allow').click()");await wait(300);
 if(await b.eval("SIM.log.filter(x=>x.kind==='action'&&x.p.action.sha==='tr3').length")!==1)fail('a second decision was sent for the same request');
 await b.eval("SIM.hang.action=false;SIM.release('action');SIM.room.tool_requests=[]");await until("document.getElementById('tool-decision').hidden",'footer gone after the slow decision settled');
 await until("!document.getElementById('tool-decision-allow').classList.contains('is-busy')",'progress bar ends once the decision settled');
 console.log('tool decision: footer replaces the direction field; allow and deny send the request id; no modal; a slow decision waits for the tool check and stays locked');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
