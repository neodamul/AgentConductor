const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const {execFileSync} = require('node:child_process');
const {pathToFileURL} = require('node:url');
const {findChrome, launch} = require('./design_parity/cdp.cjs');

(async () => {
  const chrome = findChrome();
  if (!chrome) process.exit(77);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-repository-busy-'));
  let browser;
  try {
    const file = path.join(dir, 'app.html');
    execFileSync(process.execPath, ['tests/ui_sim/build.cjs', process.cwd(), file]);
    browser = await launch(chrome, {width:1280, height:900});
    await browser.goto(pathToFileURL(file).href);
    const result = await browser.eval(`(async () => {
      const q = id => document.getElementById(id);
      const until = async check => {
        for (let i=0; i<180 && !check(); i++) await new Promise(r=>setTimeout(r,35));
        if (!check()) throw Error('Waiting for repository contention UI');
      };
      SIM.home.proposal=SIM.proposal('ready');
      SIM.home.repository_busy=true;
      const message=acMessage('native.repository_action_finishing');
      await until(()=>q('start-regenerate').disabled&&q('start-regenerate').title===message);
      if (!q('start-status').textContent.includes(message)) throw Error('Busy status explanation missing');
      q('start-regenerate').click();
      if (SIM.log.some(entry=>entry.kind==='prepare')) throw Error('The disabled plan action sent a request');
      SIM.home.repository_busy=false;
      await until(()=>!q('start-regenerate').disabled);

      SIM.room.coordinator_busy=true;
      SIM.room.report.state='waiting_approval';
      SIM.home.run_state='waiting_approval';
      SIM.home.run_available=true;
      await until(()=>!q('start-tab-tasks').disabled);
      q('start-tab-tasks').click();
      await until(()=>q('quota-refresh').disabled&&q('quota-refresh').title===message);
      q('quota-refresh').click();
      if (SIM.log.some(entry=>entry.p?.action?.kind==='quota')) throw Error('Busy quota requested');
      SIM.room.coordinator_busy=false;
      await until(()=>!q('quota-refresh').disabled);
      return true;
    })()`);
    assert.equal(result, true);
  } finally {
    if (browser) await browser.close();
    fs.rmSync(dir, {recursive:true, force:true});
  }
})().catch(error => { console.error(error); process.exitCode=1; });
