// GitHub #697: every brand avatar copy owns its gradient ids; with duplicated ids a url(#id) resolves to the first copy, and when that one sits in a
// hidden panel or was removed the other marks lose their fill (the Codex tile turned blank).
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('brand avatar ids skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-brand-ids-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 await b.goto(pathToFileURL(path.join(dir,'app.html')).href);await wait(1500);
 const result=await b.eval(`(()=>{const a=acBrandAvatar('codex','x'),c=acBrandAvatar('codex','x');const hidden=document.createElement('div');hidden.style.display='none';hidden.append(a);document.body.append(hidden,c);
  const ids=[...document.querySelectorAll('svg [id]')].map(e=>e.id),refs=[...c.querySelectorAll('*')].flatMap(e=>[...e.attributes].map(x=>x.value)).flatMap(v=>[...v.matchAll(/url\\(#([^)]+)\\)/g)].map(m=>m[1]));
  return {refs,duplicated:ids.filter((id,i)=>ids.indexOf(id)!==i),own:refs.every(id=>c.querySelector('[id="'+id+'"]'))};})()`);
 if(!result.refs.length)throw Error('the codex mark no longer references a gradient; update this test');
 if(result.duplicated.length||!result.own)throw Error('brand avatar copies share gradient ids: '+JSON.stringify(result));
 console.log('brand avatar ids: each copy keeps its own gradient');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
