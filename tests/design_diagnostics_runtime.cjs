const assert = require('node:assert/strict');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
const {describe}=require('./design_parity/diagnostics.cjs');
(async()=>{
 const chrome=findChrome();if(!chrome){console.log('diagnostics skipped: no Chrome');return;}
 const browser=await launch(chrome);
 try{
  await browser.goto('data:text/html,'+encodeURIComponent('<button id="sample" style="display:flex;font:11.5px Arial">Text<!-- ignored --><span>Mixed 한글</span></button>'));
  const before=await browser.eval('document.body.innerHTML');
  const data=await describe(browser,'#sample');
  assert.equal(data.element.tag,'BUTTON');assert.equal(data.children.length,2);
  assert.equal(data.children[0].tag,'#text');assert.equal(data.children[1].style.fontSize,'11.5px');
  assert.ok(data.element.bounds.width>0);assert.equal(data.dpr,2);
  const fonts=await browser.fonts('#sample');assert.ok(fonts.some(f=>f.glyphCount>0));
  assert.equal(await describe(browser,'#missing'),null);
  assert.equal(await browser.eval('document.body.innerHTML'),before,'evidence collection does not change the DOM');
  console.log('design diagnostics: bounds, children, real fonts and read-only behavior pass');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
