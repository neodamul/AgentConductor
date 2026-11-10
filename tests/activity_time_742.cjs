const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const { pathToFileURL } = require('node:url');
const { findChrome, launch } = require('./design_parity/cdp.cjs');

(async () => {
  const chrome = findChrome(); assert(chrome, 'Chrome is required');
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-742-'));
  let browser;
  try {
    const file = path.join(dir, 'app.html');
    execFileSync(process.execPath, ['tests/ui_sim/build.cjs', process.cwd(), file]);
    browser = await launch(chrome, { width: 1280 });
    await browser.goto(pathToFileURL(file).href);
    const preset = "SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.tasks[0].started_ms=1700000000000;";
    await browser.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(preset)});location.reload()`);
    const result = await browser.eval(`(async () => {
      for (let n=0;n<200 && !document.querySelector('#start-step-tabs [role=tab]');n++) await new Promise(r=>setTimeout(r,50));
      document.querySelectorAll('#start-step-tabs [role=tab]')[3].click();
      for (let n=0;n<200 && !document.querySelector('#activity .event.plan .event-seq');n++) await new Promise(r=>setTimeout(r,50));
      return { text: document.querySelector('#activity .event.plan .event-seq')?.textContent,
        expected: new Date(1700000000000).toLocaleTimeString([],{hour:'2-digit',minute:'2-digit',hour12:false}) };
    })()`);
    assert.equal(result.text, result.expected, 'assigned task must show its actual start time');
    const missing = await browser.eval(`(async () => {
      SIM.room.report.tasks[0].started_ms = null;
      document.dispatchEvent(new Event('visibilitychange'));
      for (let n=0;n<200;n++) {
        const text = document.querySelector('#activity .event.plan .event-seq')?.textContent;
        if (text === '') return true;
        await new Promise(r=>setTimeout(r,50));
      }
      return false;
    })()`);
    assert(missing, 'a missing start time must not show a fabricated time');
  } finally {
    if (browser) await browser.close();
    fs.rmSync(dir, { recursive: true, force: true });
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
