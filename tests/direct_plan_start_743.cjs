const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const { pathToFileURL } = require('node:url');
const { findChrome, launch } = require('./design_parity/cdp.cjs');

(async () => {
  const chrome = findChrome();
  assert(chrome, 'Chrome is required to verify direct plan start');
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-743-'));
  let browser;
  try {
    const file = path.join(dir, 'app.html');
    execFileSync(process.execPath, ['tests/ui_sim/build.cjs', process.cwd(), file]);
    browser = await launch(chrome, { width: 1280 });
    await browser.goto(pathToFileURL(file).href);
    const result = await browser.eval(`(async () => {
      const waitFor = async predicate => {
        for (let n = 0; n < 200; n++) {
          if (predicate()) return true;
          await wait(50);
        }
        return false;
      };
      await waitFor(() => document.querySelectorAll('.startup-profile').length);
      SIM.reply.baseline_status = () => ({ source_mode: 'folder', clean: true });
      SIM.reply.prepare = () => { SIM.hang.snapshot = true; setTimeout(() => { SIM.hang.snapshot = false; SIM.release('snapshot'); }, 1200); return { phase: 'ready' }; };
      document.getElementById('start-tab-team').click();
      document.getElementById('start-prepare').click();
      const started = await waitFor(() => SIM.log.some(x => x.kind === 'prepare') && !document.getElementById('start-read-plan').disabled);
      return { started, count: SIM.log.filter(x => x.kind === 'prepare').length, dialogOpen: document.getElementById('start-planning-confirm')?.open || false };
    })()`);
    assert.equal(result.dialogOpen, false, 'planning must not require a second confirmation');
    assert.equal(result.started, true, 'planning should start and settle after delayed refresh');
    assert.equal(result.count, 1, 'planning must start exactly once');
    await browser.goto(pathToFileURL(file).href);
    const stale = await browser.eval(`(async () => {
      for (let n = 0; n < 200 && !document.querySelectorAll('.startup-profile').length; n++) await wait(50);
      SIM.reply.baseline_status = () => {
        const goal = document.getElementById('start-goal');
        goal.value = 'Changed while checking the baseline';
        goal.dispatchEvent(new Event('input'));
        return { source_mode: 'folder', clean: true };
      };
      document.getElementById('start-tab-team').click();
      document.getElementById('start-prepare').click();
      for (let n = 0; n < 200; n++) {
        if (document.getElementById('start-notice').textContent === acMessage('startup.planning_confirmation_changed')) return true;
        await wait(50);
      }
      return false;
    })()`);
    assert.equal(stale, true, 'a changed goal must stop planning after the baseline check');
  } finally {
    if (browser) await browser.close();
    fs.rmSync(dir, { recursive: true, force: true });
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
