// #420: a deliberately failing behavior assertion must execute at both reference widths.
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),os=require('node:os'),{spawnSync}=require('node:child_process');
const vm=require('node:vm'),scale=require('./design_parity/spec.cjs').find(s=>s.name==='type-scale-workspace').assert;
for(const [width,version,font,expected]of [[840,true,9,true],[1280,true,9,false],[840,false,9,false],[840,false,10,true]]){
 const e={id:'fixture',className:version?'app-version':'body',childNodes:[{nodeType:3,textContent:'fixture'}],getBoundingClientRect:()=>({width:20,height:10}),matches:s=>s==='.app-version'&&version};
 const actual=vm.runInNewContext(scale,{innerWidth:width,document:{querySelectorAll:()=>[e]},getComputedStyle:()=>({fontSize:String(font)}),window:{}});
 assert.equal(actual,expected,'only the approved narrow version can use 9px');
}
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-parity-assertion-'));
try{
 const hook=path.join(dir,'hook.cjs');
 fs.writeFileSync(hook,`const Module=require('node:module'),path=require('node:path'),load=Module._load;
 global.setTimeout=(fn,_delay,...args)=>setImmediate(fn,...args);
 Module._load=function(name,parent,...rest){
  if(path.basename(parent?.filename||'')==='parity.cjs'){
   if(name==='./spec.cjs')return [{name:'forced-behavior-failure',assert:'false',app:{}}];
   if(name==='./cdp.cjs')return {findChrome:()=> 'fixture',launch:async()=>({goto:async()=>{},eval:async()=>false,viewport:async()=>{},close:async()=>{}})};
  }
  return load.call(this,name,parent,...rest);
 };`);
 for(const theme of ['light','dark'])for(const width of [840,1280]){
  const result=spawnSync(process.execPath,['--require',hook,'tests/design_parity/parity.cjs','--out',path.join(dir,theme+'-'+width)],{env:{...process.env,PARITY_THEME:theme,PARITY_WIDTH:String(width)},encoding:'utf8',timeout:15000});
  assert.equal(result.status,1,theme+'/'+width+' must execute and reject the false assertion: '+result.stdout+result.stderr);
  assert.match(result.stdout,/forced-behavior-failure: assertion failed/);
 }
 console.log('Both themes and both widths execute behavior assertions instead of silently skipping them.');
}finally{fs.rmSync(dir,{recursive:true,force:true});}
