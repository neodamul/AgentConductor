// #409 / #431: exercise the real runner with identical page scripts at different entry viewports.
const assert=require('node:assert/strict'),fs=require('node:fs'),os=require('node:os'),path=require('node:path');
const {execFileSync}=require('node:child_process'),{findChrome}=require('./design_parity/cdp.cjs');
const {decode}=require('./design_parity/png.cjs');
if(!findChrome()){console.log('parity viewports skipped: no Chrome');process.exit(0);}
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-entry-viewports-'));
try{
 const run=`const e=document.createElement('div');e.id='viewport-probe';e.style.cssText='position:fixed;left:20px;top:20px;background:red;width:'+(innerWidth/10)+'px;height:'+(innerHeight/10)+'px';document.body.append(e);`;
 const spec=(name,viewport)=>({name,viewport,props:[],mock:{selector:'#viewport-probe',run},app:{selector:'#viewport-probe',run}});
 const specs=[spec('object',{width:840,height:760}),spec('array',[560,720]),spec('default')];
 for(const [name,viewport,w,h] of [['assert-object',{width:840,height:760},840,760],['assert-array',[560,720],560,720]])
  specs.push({name,viewport,app:{},assert:`innerWidth===${w}&&innerHeight===${h}`});
 const preload=path.join(dir,'spec.cjs'),modulePath=require.resolve('./design_parity/spec.cjs');
 fs.writeFileSync(preload,`require(${JSON.stringify(modulePath)});require.cache[${JSON.stringify(modulePath)}].exports=${JSON.stringify(specs)};`);
 const output=execFileSync(process.execPath,['--require',preload,'tests/design_parity/parity.cjs','--out',dir],{env:{...process.env,PARITY_WIDTH:'1110'},encoding:'utf8'});
 assert.match(output,/5 element\(s\), 0 difference\(s\)/);
 for(const [name,w,h] of [['object',168,152],['array',112,144],['default',222,192]])for(const side of ['mock','app']){
  const png=decode(fs.readFileSync(path.join(dir,`${name}.${side}.png`)));
  assert.equal(png.width,w,`${name}/${side} uses its own width`);
  assert.equal(png.height,h,`${name}/${side} uses its own height`);
 }
 console.log('parity viewports: object/array/default capture sizes, page-state cache and assertions pass');
}finally{fs.rmSync(dir,{recursive:true,force:true});}
