// #435: fallback font line boxes must size the import row instead of being clipped to 26px.
const assert=require('node:assert/strict'),fs=require('node:fs'),os=require('node:os'),path=require('node:path');
const {execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{
 const chrome=findChrome();if(!chrome){console.log('import row skipped: no Chrome');return;}
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-import-row-'));let browser;
 try{
  const html=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),html]);
  browser=await launch(chrome);await browser.goto(pathToFileURL(html).href);
  for(const size of [15.5,22]){
   const box=await browser.eval(`(()=>{
    const row=document.getElementById('start-mcp-controls'),preview=document.getElementById('start-mcp-preview');
    preview.style.lineHeight='${size}px';
    const style=getComputedStyle(preview),buttons=[...row.querySelectorAll('button')];
    return {expected:${size}+parseFloat(style.paddingTop)+parseFloat(style.paddingBottom)+parseFloat(style.borderTopWidth)+parseFloat(style.borderBottomWidth),row:row.getBoundingClientRect().height,heights:buttons.map(b=>b.getBoundingClientRect().height)};
   })()`);
   assert.equal(box.row,box.expected,'row expands for a taller fallback text line');
   assert.ok(box.heights.every(h=>h===box.row),'all import buttons stretch to the tallest label');
  }
  console.log('import row: taller fallback labels expand every sibling without a fixed height');
 }finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}
})().catch(e=>{console.error(e);process.exitCode=1;});
