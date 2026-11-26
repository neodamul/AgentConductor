const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const {execFileSync} = require('node:child_process');
const {pathToFileURL} = require('node:url');
const {findChrome, launch} = require('./design_parity/cdp.cjs');

const chrome = findChrome();
if (!chrome) process.exit(77);
const root = path.resolve(__dirname, '..');
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-goal-reference-list-'));
const page = path.join(dir, 'app.html');
const preset = `SIM.home.slugs=Array.from({length:12}, (_, i)=>({
  slug:'goal_'+(i+1), goal:'Fixture '+(i+1), active:i===0, created_ms:Date.UTC(2026,10,26-i),
  progress:{stage:i===0?'completed':'no_plan'}, references:[
    {kind:'jira',id:'EMM-'+(i+1)},
    {kind:'confluence',id:'123456789012345678901234567890'+String(i+1).padStart(2,'0')}
  ]
})).reverse();`;
const get = id => "document.getElementById(" + JSON.stringify('start-' + id) + ")";
async function waitFor(browser, condition) {
  for (let i = 0; i < 100; i++) {
    if (await browser.eval(condition)) return;
    await new Promise(resolve => setTimeout(resolve, 25));
  }
  assert.fail('Timed out waiting for ' + condition);
}
async function main() {
  let browser;
  try {
    execFileSync('node', [path.join(root, 'tests/ui_sim/build.cjs'), root, page]);
    browser = await launch(chrome, {width:520, height:900});
    const url = pathToFileURL(page).href;
    await browser.goto(url);
    await browser.eval(`sessionStorage.setItem('SIM_PRESET', ${JSON.stringify(preset)})`);
    await browser.goto(url);
    const evalJs = script => browser.eval(script);
    const click = id => evalJs(get(id) + '.click()');
    const rows = "Array.from(document.querySelectorAll('#start-goal-list .goal-item strong'), n=>n.textContent)";
    const input = (id, val, event) => evalJs(`(() => { const el=${get(id)}; el.value=${JSON.stringify(val)}; el.dispatchEvent(new Event('${event}', {bubbles:true})); })()`);

    await waitFor(browser, get('goal-count') + '.textContent==="12"');
    await click('goal-list-open');
    assert.equal((await evalJs(rows)).length, 10);
    assert.deepEqual((await evalJs(rows)).slice(0,2), ['goal_1','goal_2']);
    assert.ok(await evalJs("!!document.querySelector('#start-goal-list time[datetime]')"));
    await input('goal-sort', 'created-asc', 'change');
    assert.deepEqual((await evalJs(rows)).slice(0,2), ['goal_12','goal_11']);
    await input('goal-sort', 'created-desc', 'change');
    assert.equal(await evalJs("document.querySelectorAll('.goal-folder').length"), 10);
    assert.equal(await evalJs("document.querySelectorAll('.goal-trash').length"), 10);
    assert.ok(await evalJs("!!document.querySelector('.goal-result')"));
    assert.ok(await evalJs(get('goal-prev') + '.disabled'));
    await click('goal-next');
    assert.deepEqual(await evalJs(rows), ['goal_11', 'goal_12']);
    assert.ok(await evalJs(get('goal-next') + '.disabled'));

    await input('goal-search', 'confluence: 12345678901234567890123456789001', 'input');
    assert.deepEqual(await evalJs(rows), ['goal_1']);
    assert.ok(await evalJs(get('goal-pagination') + '.hidden'));
    assert.match(await evalJs("document.querySelector('.goal-reference-ids').textContent"), /EMM-1/);
    await input('goal-search', 'missing reference', 'input');
    assert.equal((await evalJs(rows)).length, 0);
    assert.match(await evalJs(get('goal-list') + '.textContent'), /검색에 해당|No Goals/);
    await input('goal-search', '', 'input');
    await input('goal-sort', 'jira-desc', 'change');
    assert.deepEqual((await evalJs(rows)).slice(0, 3), ['goal_12', 'goal_11', 'goal_10']);
    await click('goals-close');
    await click('goal-list-open');
    assert.equal((await evalJs(rows))[0], 'goal_12');
    for (const theme of ['light', 'dark']) {
      assert.ok(await evalJs(`(() => {
        document.documentElement.dataset.theme='${theme}';
        const box=${get('goals')};
        return box.scrollWidth <= box.clientWidth + 1;
      })()`), 'Goal list should not scroll horizontally in ' + theme);
    }
    console.log('Goal list browser: search, sort, paging, retained actions, and narrow light/dark panel passed');
  } finally {
    if (browser) await browser.close();
    fs.rmSync(dir, {recursive:true, force:true});
  }
}
main().catch(error => { console.error(error); process.exitCode = 1; });
