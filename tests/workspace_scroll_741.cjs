const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const { pathToFileURL } = require('node:url');
const { findChrome, launch } = require('./design_parity/cdp.cjs');

(async () => {
  const chrome = findChrome();
  assert(chrome, 'Chrome is required');
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-741-'));
  let browser;
  try {
    const file = path.join(dir, 'app.html');
    execFileSync(process.execPath, ['tests/ui_sim/build.cjs', process.cwd(), file]);
    for (const width of [840, 1280]) {
      browser = await launch(chrome, { width, height: 892 });
      await browser.goto(pathToFileURL(file).href);
      const result = await browser.eval(`(async () => {
        for (let n = 0; n < 200 && !document.getElementById('start-goal'); n++) await wait(50);
        const body = document.getElementById('start-workspace-body');
        if (!body) return { exists: false };
        document.getElementById('start-goal').style.height = '360px';
        const footer = document.querySelector('#start-panel-workspace .startup-actions');
        const top = footer.getBoundingClientRect().top;
        body.scrollTo({top: body.scrollHeight, behavior: 'instant'});
        await wait(100);
        const row = document.getElementById('start-mcp-controls').getBoundingClientRect();
        return { exists: true, overflow: body.scrollHeight > body.clientHeight,
          rowVisible: row.height > 0 && row.bottom <= footer.getBoundingClientRect().top,
          footerFixed: Math.abs(top - footer.getBoundingClientRect().top) < 1,
          width: innerWidth, scrollTop: body.scrollTop, scrollHeight: body.scrollHeight,
          clientHeight: body.clientHeight, rowTop: row.top, rowBottom: row.bottom,
          footerTop: footer.getBoundingClientRect().top };
      })()`);
      assert(result.exists, `workspace body must be independently scrollable: ${JSON.stringify(result)}`);
      assert(result.overflow, `a tall goal must overflow the body: ${JSON.stringify(result)}`);
      assert(result.rowVisible, `Goal tools must be visible above the footer: ${JSON.stringify(result)}`);
      assert(result.footerFixed, `footer must remain stationary: ${JSON.stringify(result)}`);
      await browser.close(); browser = null;
    }
  } finally {
    if (browser) await browser.close();
    fs.rmSync(dir, { recursive: true, force: true });
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
