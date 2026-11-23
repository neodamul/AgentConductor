const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const { pathToFileURL } = require('node:url');
const { findChrome, launch } = require('./design_parity/cdp.cjs');

(async () => {
  const chrome = findChrome();
  if (!chrome) { process.exitCode = 77; return; }
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-chat-markdown-'));
  let browser;
  try {
    const app = path.join(dir, 'app.html');
    execFileSync(process.execPath, ['tests/ui_sim/build.cjs', process.cwd(), app]);
    browser = await launch(chrome, { width: 840, height: 900 });
    const url = pathToFileURL(app).href;
    await browser.goto(url);
    const html = '<h1>Result</h1><p><a href="https://example.test/docs">Docs</a></p>' +
      '<div class="manual-table"><table><tr>' + '<td>long heading</td>'.repeat(25) +
      '</tr></table></div><pre><code>' + 'wideCode'.repeat(100) +
      '</code></pre><p>&lt;img src=x onerror=bad()&gt;</p>';
    const preset = 'SIM.reply.chat_history=()=>({turns:[{role:"assistant",text:"# Result",html:' + JSON.stringify(html) + '}]})';
    for (const theme of ['light', 'dark']) {
      await browser.eval('sessionStorage.setItem("SIM_PRESET",' + JSON.stringify(preset) + ')');
      await browser.goto(url);
      const checks = await browser.eval(`(async()=>{
        document.documentElement.dataset.theme=${JSON.stringify(theme)};
        const button=document.getElementById('start-chat-button');
        const panel=document.getElementById('start-chat');
        const find=selector=>panel.querySelector(selector);
        for(let i=0;i<100&&(panel.hidden||!find('.chat-markdown'));i++){
          if(panel.hidden&&!button.disabled)button.click();
          await wait(30);
        }
        if(panel.hidden||!find('.chat-markdown'))throw Error('Restored reply did not render');
        const wide=selector=>find(selector).scrollWidth>find(selector).clientWidth;
        if (!wide('.manual-table') || !wide('pre') || panel.scrollWidth !== panel.clientWidth) {
          const dimensions = ['.manual-table','pre','.chat-bubble'].map(selector=>
            [selector,find(selector).scrollWidth,find(selector).clientWidth]);
          throw Error('Unexpected scroll dimensions: '+JSON.stringify([...dimensions,['panel',panel.scrollWidth,panel.clientWidth]]));
        }
        const noAutomaticNavigation=!SIM.log.some(e=>e.kind==='open_chat_link');
        find('.chat-markdown a').click();
        return {
          title:find('.chat-markdown h1').textContent,
          rawSafe:!find('.chat-markdown img'),
          internalScroll:wide('.manual-table')&&wide('pre')&&panel.scrollWidth===panel.clientWidth,
          noAutomaticNavigation,
          opened:SIM.log.some(e=>e.kind==='open_chat_link'&&e.p.url==='https://example.test/docs')
        };
      })()`);
      assert.deepEqual(checks, {
        title:'Result', rawSafe:true, internalScroll:true, noAutomaticNavigation:true, opened:true
      }, theme);
    }
    console.log('Coordinator Markdown retains the chat layout, scrolls wide content and opens links on click.');
  } finally {
    try { await browser?.close(); } finally { fs.rmSync(dir, { recursive: true, force: true }); }
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
