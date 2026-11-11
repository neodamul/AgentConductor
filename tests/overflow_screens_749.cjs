const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const { pathToFileURL } = require('node:url');
const { findChrome, launch } = require('./design_parity/cdp.cjs');

(async () => {
  const chrome = findChrome(); assert(chrome, 'Chrome is required');
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-749-screens-')); let browser;
  try {
    const file = path.join(dir, 'app.html');
    execFileSync(process.execPath, ['tests/ui_sim/build.cjs', process.cwd(), file]);
    browser = await launch(chrome, { width: 840, height: 892 });
    await browser.goto(pathToFileURL(file).href);
    const preset = "SIM.home.run_available=true;SIM.home.run_state=null;SIM.home.proposal=SIM.proposal('ready');SIM.home.config.goal='A very long Goal title which must not be lost when it is truncated '.repeat(5);";
    await browser.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(preset)});location.reload()`);
    const report = await browser.eval(`(async () => {
      const delay=()=>new Promise(r=>setTimeout(r,80));
      for(let n=0;n<200&&(!document.querySelector('#start-tab-workspace')?.onclick||!SIM.log.some(x=>x.kind==='snapshot'));n++)await delay();
      const errors=[];let checked=0;
      for(let step=0;step<4;step++) {
        const tab=document.querySelectorAll('#start-step-tabs [role=tab]')[step];if(step===3)document.querySelector('#start-open').click();else tab.click();const reached=()=>step===3?document.body.classList.contains('on-run'):document.querySelectorAll('#start-step-tabs [role=tab]')[step]?.getAttribute('aria-selected')==='true';for(let n=0;n<100&&!reached();n++)await delay();if(!reached()){errors.push({step,error:'step did not become selected'});continue;}
        for(const el of document.querySelectorAll('body *')) {
          const style=getComputedStyle(el), rect=el.getBoundingClientRect();
          if(!rect.width||!rect.height||style.visibility==='hidden'||style.display==='none')continue;
          if(style.textOverflow!=='ellipsis'&&!(parseInt(style.webkitLineClamp,10)>0))continue;
          if(el.scrollWidth<=el.clientWidth+1&&el.scrollHeight<=el.clientHeight+1)continue;
          const text=el instanceof HTMLInputElement?(el.type==='password'?'':el.value.trim()):el.textContent.trim();
          if(!text)continue;
          el.dispatchEvent(new PointerEvent('pointerover',{bubbles:true}));checked++;
          if(!el.title.includes(text))errors.push({step,tag:el.tagName,id:el.id,text:text.slice(0,80),title:el.title.slice(0,80)});
        }
      }
      return {checked,errors};
    })()`);
    assert(report.checked > 0, 'screen walk must inspect actual truncated text');
    assert.deepEqual(report.errors, [], JSON.stringify(report.errors));
  } finally { if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true}); }
})().catch(error=>{console.error(error);process.exitCode=1;});
