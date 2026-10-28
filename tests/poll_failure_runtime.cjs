// GitHub #221: a background poll that times out must not raise a toast on its own, and a page-side timeout is
// reported to the native side so the diagnostics log shows it.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const source=fs.readFileSync('src/ui/startup.js','utf8'),room=fs.readFileSync('src/ui/room.js','utf8');
const notices=[];
const context={notice:(text,error)=>notices.push({text,error}),acMessage:key=>key};
vm.createContext(context);
const a=source.indexOf('let pollFailures'),b=source.indexOf('\n',a);
assert(a>=0&&b>a,'startup.js defines pollFailed');
vm.runInContext(source.slice(a,b),context);
const fail=()=>vm.runInContext('pollFailed(Object.assign(new Error("Request timed out (snapshot)"),{transport:true}),true)',context);
fail();fail();
assert.equal(notices.length,0,'one or two missed polls are retried silently');
fail();
assert.equal(notices.length,1,'the third miss in a row is shown');
assert.equal(notices[0].error,true);
vm.runInContext('pollSucceeded()',context);
fail();fail();
assert.equal(notices.length,1,'a success starts the count again');
vm.runInContext('pollFailed(Object.assign(new Error("Request timed out (snapshot)"),{transport:true}),false)',context);
assert.equal(notices.length,2,'an explicit refresh timeout is shown immediately');
vm.runInContext('pollFailed(new Error("a real error"),true)',context);
assert.equal(notices.length,3,'an error that is not a missed reply shows at once');
assert(room.includes('pollFailed(e,background)')&&room.includes('pollSucceeded()'),'the Agent Room poll follows the same rule');

(async()=>{
  const sent=[];
  const w={__acRequest:async text=>{const packet=JSON.parse(text);sent.push(packet);return packet.kind==='client_timeout'?'{}':new Promise(()=>{});}};
  const ctx={window:w,acMessage:(key,args)=>key+(args?args.value1:''),document:{hidden:false,addEventListener(){},removeEventListener(){}},performance,setTimeout,clearTimeout,Date,JSON,Error,Promise};
  vm.createContext(ctx);
  const s=source.indexOf('async function request('),e=source.indexOf('\n',s);
  vm.runInContext(source.slice(s,e),ctx);
  await assert.rejects(vm.runInContext('request({kind:"snapshot"},20)',ctx),/snapshot/);
  await new Promise(r=>setTimeout(r,10));
  const report=sent.find(p=>p.kind==='client_timeout');
  assert(report,'the page tells the native side that it gave up');
  assert.equal(report.of,'snapshot');
  assert(report.waited>=20&&typeof report.hidden==='boolean');
  assert(room.includes("kind:'client_timeout'")||room.includes('kind:"client_timeout"'),'the Agent Room request reports too');
  console.log('A missed poll is retried quietly, shown after three, and page timeouts reach the diagnostics log.');
})().catch(error=>{console.error(error);process.exitCode=1;});
