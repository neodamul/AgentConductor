const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm'),{EventEmitter}=require('node:events');
const source=fs.readFileSync(require.resolve('./design_parity/chrome_process.cjs'),'utf8');
function load(platform,run=()=>{throw Error('unexpected taskkill');}){const module={exports:{}};vm.runInNewContext(source,{module,process:{platform},require:()=>({spawnSync:run}),setTimeout:fn=>{queueMicrotask(fn);return 1;},clearTimeout:()=>{}});return module.exports.stopChrome;}
function child(){const p=new EventEmitter();Object.assign(p,{pid:2468,exitCode:null,signalCode:null,kill(){p.exitCode=0;p.emit('exit',0);}});return p;}
(async()=>{
 const dead=child();dead.exitCode=0;dead.kill=()=>{throw Error('already dead');};await load('darwin')(dead);
 const live=child();await load('darwin')(live);assert.equal(live.listenerCount('exit'),0);
 const win=child();let calls=0;await load('win32',(cmd,args,options)=>{calls++;assert.equal(cmd,'taskkill');assert.deepEqual([...args],['/PID','2468','/T','/F']);assert.equal(options.timeout,5000);assert.equal(options.windowsHide,true);win.exitCode=0;win.emit('exit',0);return {status:0};})(win);assert.equal(calls,1);
 const stuck=child();stuck.kill=()=>false;await assert.rejects(load('darwin')(stuck),/did not exit/);assert.equal(stuck.listenerCount('exit'),0);
 const denied=child();await assert.rejects(load('win32',()=>({status:1}))(denied),/Cannot stop owned/);assert.equal(denied.listenerCount('exit'),0);
 const gone=child();await load('win32',()=>{queueMicrotask(()=>{gone.exitCode=0;gone.emit('exit',0);});return {status:128};})(gone);assert.equal(gone.listenerCount('exit'),0);
 const late=child();await load('win32',()=>{queueMicrotask(()=>{late.exitCode=0;late.emit('exit',0);});return {status:255};})(late);assert.equal(late.listenerCount('exit'),0);
 const missing=child();missing.pid=undefined;await assert.rejects(load('win32')(missing),/no owned process/);
 console.log('Chrome teardown: exited/live/tree/already-gone/timeout/failure/ownership checks pass');
})().catch(e=>{console.error(e);process.exitCode=1;});
