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
        body.scrollTop = body.scrollHeight;
        await wait(50);
        const row = document.getElementById('start-mcp-controls').getBoundingClientRect();
        return { exists: true, overflow: body.scrollHeight > body.clientHeight,
          rowVisible: row.height > 0 && row.bottom <= footer.getBoundingClientRect().top,
          footerFixed: Math.abs(top - footer.getBoundingClientRect().top) < 1 };
      })()`);
      assert(result.exists, 'workspace body must be independently scrollable');
      assert(result.overflow, 'a tall goal must overflow the body');
      assert(result.rowVisible, 'Goal tools must be visible above the footer');
      assert(result.footerFixed, 'footer must remain stationary');
      await browser.close(); browser = null;
    }
  } finally {
    if (browser) await browser.close();
    fs.rmSync(dir, { recursive: true, force: true });
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
