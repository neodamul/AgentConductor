// Stop only the browser process we spawned; never target the operator's other Chrome windows.
const {spawnSync}=require('node:child_process');
async function stopChrome(proc){
 if(proc.exitCode!==null||proc.signalCode!==null)return;
 if(!Number.isInteger(proc.pid)||proc.pid<=0)throw Error('Chrome has no owned process ID');
 let timer,onExit;
 const gone=new Promise(resolve=>{onExit=resolve;proc.once('exit',onExit);});
 try{
  if(process.platform==='win32'){
   const result=spawnSync('taskkill',['/PID',String(proc.pid),'/T','/F'],{windowsHide:true,stdio:'ignore',timeout:5000});
   if((result.error||result.status!==0)&&proc.exitCode===null&&proc.signalCode===null)throw Error('Cannot stop owned Chrome process tree: '+(result.error?.message||result.status));
  }else proc.kill();
  await Promise.race([gone,new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('Owned Chrome did not exit within 5000ms')),5000);})]);
 }finally{clearTimeout(timer);proc.removeListener('exit',onExit);}
}
module.exports={stopChrome};
