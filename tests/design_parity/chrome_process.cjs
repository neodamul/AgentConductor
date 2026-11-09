// Stop only the browser process we spawned; never target the operator's other Chrome windows.
const {spawnSync}=require('node:child_process');
async function stopChrome(proc){
 if(proc.exitCode!==null||proc.signalCode!==null)return;
 if(!Number.isInteger(proc.pid)||proc.pid<=0)throw Error('Chrome has no owned process ID');
 let timer,onExit,killed=false;
 const gone=new Promise(resolve=>{onExit=resolve;proc.once('exit',onExit);});
 try{
  if(process.platform==='win32'){
   // taskkill exits non-zero (128, 255, ...) when the tree is already going away; give the exit event a moment before failing.
   const result=spawnSync('taskkill',['/PID',String(proc.pid),'/T','/F'],{windowsHide:true,stdio:'ignore',timeout:5000});
   killed=!result.error&&result.status===0;
   if((result.error||result.status!==0)&&proc.exitCode===null&&proc.signalCode===null){
    await Promise.race([gone,new Promise(resolve=>{timer=setTimeout(resolve,2000);})]);
    clearTimeout(timer);
    if(proc.exitCode===null&&proc.signalCode===null)throw Error('Cannot stop owned Chrome process tree: '+(result.error?.message||result.status));
   }
  }else proc.kill();
  // Windows reports the exit event late after taskkill /T /F succeeded; the tree is gone, so waiting longer is enough.
  await Promise.race([gone,new Promise((resolve,reject)=>{timer=setTimeout(()=>killed?resolve():reject(Error('Owned Chrome did not exit within 5000ms')),killed?20000:5000);})]);
 }finally{clearTimeout(timer);proc.removeListener('exit',onExit);}
}
module.exports={stopChrome};
