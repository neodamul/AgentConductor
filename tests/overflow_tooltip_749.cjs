const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const { pathToFileURL } = require('node:url');
const { findChrome, launch } = require('./design_parity/cdp.cjs');
(async () => {
  const chrome = findChrome(); assert(chrome, 'Chrome is required');
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-749-')); let browser;
  try {
    const file = path.join(dir, 'app.html');
    execFileSync(process.execPath, ['tests/ui_sim/build.cjs', process.cwd(), file]);
    browser = await launch(chrome, { width: 840 });
    await browser.goto(pathToFileURL(file).href);
    const values = await browser.eval(`(async () => {
      for (let n=0;n<200 && !document.querySelector('.startup-main');n++) await new Promise(r=>setTimeout(r,50));
      const el=document.createElement('div');el.textContent='A deliberately long task title that cannot fit';
      el.style.cssText='width:45px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap';
      document.querySelector('.startup-main').append(el);
      const hover=()=>el.dispatchEvent(new PointerEvent('pointerover',{bubbles:true}));
      hover();const truncated=el.title;
      el.style.width='600px';hover();const expanded=el.title;
      el.title='Original description';el.style.width='45px';hover();const existing=el.title;
      const input=document.createElement('input');input.value='A long workspace path that cannot fit';input.style.cssText='width:45px;overflow:hidden;text-overflow:ellipsis';document.querySelector('.startup-main').append(input);input.dispatchEvent(new PointerEvent('pointerover',{bubbles:true}));
      const password=document.createElement('input');password.type='password';password.value='secret-that-must-not-appear';password.style.cssText='width:20px;overflow:hidden;text-overflow:ellipsis';document.querySelector('.startup-main').append(password);password.dispatchEvent(new PointerEvent('pointerover',{bubbles:true}));
      return {truncated,expanded,existing,text:el.textContent,inputTitle:input.title,inputValue:input.value,passwordTitle:password.title};
    })()`);
    assert.equal(values.passwordTitle, '', 'password values must not become tooltips');
    assert.equal(values.inputTitle, values.inputValue, 'truncated input values must be available');
    assert.equal(values.truncated, values.text, 'truncated text must have its full content');
    assert.equal(values.expanded, '', 'generated tooltip must clear when text fits');
    assert.equal(values.existing, 'Original description\n' + values.text, 'existing descriptions and full clipped text must both remain available');
  } finally { if(browser) await browser.close(); fs.rmSync(dir,{recursive:true,force:true}); }
})().catch(error=>{console.error(error);process.exitCode=1;});
