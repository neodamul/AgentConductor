// #435: exercise the real CDP driver with a launcher that has already exited.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm'),path=require('node:path'),{EventEmitter}=require('node:events');
const source=fs.readFileSync('tests/design_parity/cdp.cjs','utf8');
function stopFor(platform,spawnSync){const module={exports:{}};vm.runInNewContext(fs.readFileSync('tests/design_parity/chrome_process.cjs','utf8'),{module,process:{platform},require:()=>({spawnSync}),setTimeout,clearTimeout});return module.exports;}
async function check(platform,alreadyExited){
 const child=new EventEmitter();Object.assign(child,{pid:2468,exitCode:alreadyExited?0:null,signalCode:null,kill(){if(child.exitCode!==null)return false;child.exitCode=0;child.emit('exit',0);return true;}});
 let removed=false,treeKills=0;const spawnSync=(cmd,args,options)=>{treeKills++;assert.equal(cmd,'taskkill');assert.deepEqual([...args],['/PID','2468','/T','/F']);assert.equal(options.timeout,5000);child.exitCode=0;child.emit('exit',0);return {status:0};};
 class WebSocket{constructor(){queueMicrotask(()=>this.onopen());}send(raw){const {id}=JSON.parse(raw);queueMicrotask(()=>this.onmessage({data:JSON.stringify({id,result:{}})}));}close(){}}
 const module={exports:{}};const fakeFs={mkdtempSync:()=>'/owned/profile',existsSync:()=>true,readFileSync:()=> '123\n',rmSync(p){assert.equal(p,'/owned/profile');removed=true;}};
 vm.runInNewContext(source,{module,WebSocket,process:{env:{},platform},setTimeout,clearTimeout,fetch:async()=>({json:async()=>[{type:'page',webSocketDebuggerUrl:'ws://owned'}]}),require:name=>{
  if(name==='child_process')return {spawn:()=>child};if(name==='fs')return fakeFs;if(name==='os')return {tmpdir:()=>'/tmp'};if(name==='path')return path;
  if(name==='./chrome_process.cjs')return stopFor(platform,spawnSync);throw Error(name);
 }});
 const browser=await module.exports.launch('owned-chrome');let timer;
 try{await Promise.race([browser.close(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('CDP teardown waited for an exit event that cannot arrive')),100);})]);}
 finally{clearTimeout(timer);}
 assert.equal(removed,true);assert.equal(child.listenerCount('exit'),0);assert.equal(treeKills,platform==='win32'&&!alreadyExited?1:0);
}
(async()=>{for(const platform of ['darwin','win32'])for(const dead of [true,false])await check(platform,dead);console.log('CDP teardown: exited and live owned launchers close on both platforms');})().catch(e=>{console.error(e);process.exitCode=1;});
