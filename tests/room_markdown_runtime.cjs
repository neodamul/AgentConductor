const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const {execFileSync} = require('node:child_process');
const {pathToFileURL} = require('node:url');
const {findChrome, launch} = require('./design_parity/cdp.cjs');

const vm = require('node:vm');
const context = {window:{}, document:{addEventListener() {}}, TextEncoder};
vm.runInNewContext(fs.readFileSync('src/ui/markdown.js','utf8'),context);
const taskLine = context.window.acMarkdown.taskLine;
for (const [source,text] of [
  ['# Plan','Plan'],['> Quote','Quote'],['1. Inspect','Inspect'],
  ['> 2. **Change** the `API`','Change the API'],
  ['**Goal:**\n<scope>\n- __Update__ the parser','Update the parser'],
  ['**Use `a__b`** with C# and snake_case','Use a__b with C# and snake_case'],
  ['**__Nested__** and unfinished **text','Nested and unfinished **text'],
  ['42 tasks and #123','42 tasks and #123'],
  ['x'.repeat(164),'x'.repeat(160)+'…'],
  ['`'.repeat(16000),'`'.repeat(160)+'…']
]) assert.equal(taskLine(source),text);

const samples = {
  plan: '# **Plan heading**\n\n**Check**\nline two',
  edited: '**Updated**\nkeep draft',
  activity: '**Message**\nsecond line\n\n[Docs](https://example.test/docs) <img src=x>',
  lesson: '- **Lesson**\n- Another'
};
const html = {
  [samples.plan]: '<h1>Plan heading</h1><p><strong>Check</strong>\nline two</p>',
  [samples.edited]: '<p><strong>Updated</strong>\nkeep draft</p>',
  [samples.activity]: '<p><strong>Message</strong>\nsecond line</p><p><a href="https://example.test/docs">Docs</a> &lt;img src=x&gt;</p>',
  [samples.lesson]: '<ul><li><strong>Lesson</strong></li><li>Another</li></ul>'
};
const raw = 'literal **raw**\nnext';
html[raw] = '<p>literal <strong>raw</strong>\nnext</p>';
const native = `const md=${JSON.stringify(html)}; SIM.reply.render_markdown=p=>({html:p.texts.map(t=>md[t]??null)});`;
(async () => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-room-markdown-'));
  let browser;
  try {
    const app = path.join(temp, 'app.html');
    execFileSync(process.execPath, ['tests/ui_sim/build.cjs', process.cwd(), app]);
    browser = await launch(findChrome(), {width:840,height:900});
    const url = pathToFileURL(app).href;
    await browser.goto(url);
    const preset = async code => {
      await browser.eval('sessionStorage.setItem("SIM_PRESET",'+JSON.stringify(native+code)+')');
      await browser.goto(url);
    };
    const until = expr => browser.eval(`(async()=>{for(let i=0;i<180;i++){if(${expr})return true;await wait(40);}throw Error('Room Markdown did not settle: '+${JSON.stringify(expr)});})()`);
    await preset(`SIM.home.proposal=SIM.proposal('ready');SIM.plan.tasks[0].prompt=${JSON.stringify(samples.plan)};`);
    await until("!!document.querySelector('#start-tasks textarea[data-key=prompt]')");
    assert.equal(await browser.eval("document.querySelector('#start-tasks .task-oneline').textContent"),'Plan heading');

    await browser.eval("const row=document.querySelector('#start-tasks details');row.open=true;row.querySelector('.plan-markdown-preview').open=true");
    await until("!!document.querySelector('.plan-markdown-preview h1')");
    assert.equal(await browser.eval("document.querySelector('.plan-markdown-preview p').textContent"),'Check\nline two');
    assert.equal(await browser.eval("document.querySelector('#start-tasks textarea[data-key=prompt]').value"),samples.plan);
    await browser.eval(`const p=document.querySelector('#start-tasks textarea[data-key=prompt]');p.value=${JSON.stringify(samples.edited)};p.dispatchEvent(new Event('input',{bubbles:true}));`);
    await until("document.querySelector('.plan-markdown-preview strong')?.textContent==='Updated'");
    assert.equal(await browser.eval("document.querySelector('#start-tasks textarea[data-key=prompt]').value"),samples.edited);

    await preset(`const text=${JSON.stringify(samples.plan)};
      SIM.home.proposal=SIM.proposal('ready');SIM.home.proposal.slug='g';SIM.home.proposal.slug_phase='p2';
      SIM.home.slugs=[{slug:'g',goal:'g',active:true,plan:{analysis:text,phases:[{id:'p1',brief:text,specs:[{...SIM.plan.tasks[0],prompt:text}]},{id:'p2',specs:[]}]},
        progress:{stage:'verification',current_phase:'p2',phases:[{id:'p1',name:'History',stage:'completed',task:['task-1'],completed:['task-1'],verification:['task-1']},{id:'p2',name:'Current',stage:'plan',task:[],completed:[],verification:[]}]}}];`);
    await browser.eval("document.getElementById('start-tab-review').click()");
    await until("!!document.querySelector('.plan-analysis-text h1')&&document.querySelectorAll('#rail-outline-body .phase-tree summary').length===2");
    await browser.eval("document.querySelector('#rail-outline-body .phase-tree summary').click()");
    await until("!!document.querySelector('.plan-phase-brief h1')&&!!document.querySelector('.task-detail .task-prompt h1')");
    await browser.eval("document.querySelector('.plan-phase-tasks li').click()");
    assert.equal(await browser.eval("document.querySelector('.plan-phase-tasks .task-detail').hidden"),false);

    for (const theme of ['light','dark']) {
      await preset(`SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.plan.tasks[0].prompt=${JSON.stringify(samples.plan)};
        SIM.reply.task=p=>({task:SIM.room.report.tasks[p.index],events:[{seq:1,payload:{type:'assistant',message:{content:[{type:'text',text:${JSON.stringify(samples.activity)}}]}}},{seq:2,payload:{type:'item.completed',item:{type:'command_execution',exit_code:0,aggregated_output:${JSON.stringify(raw)}}}}],stdout:${JSON.stringify(raw)},stderr:'',knowledge:[],step_lessons:[{task:'task-1',cause:'Reason',response:${JSON.stringify(samples.lesson)}}]});`);
      await browser.eval(`document.documentElement.dataset.theme='${theme}'`);
      await until("!!document.querySelector('#activity .event-text h1')&&!!document.querySelector('#activity .room-markdown a')&&!!document.querySelector('#lessons li strong')");
      assert.equal(await browser.eval("document.getElementById('selected-agent').textContent"),'Plan heading');
      assert.match(await browser.eval("document.getElementById('inspector-context').textContent"),/ · Plan heading$/);
      assert.equal(await browser.eval("document.querySelector('.outline-task-heading strong').textContent"),'T1 Plan heading');

      assert.equal(await browser.eval("document.querySelector('#activity .room-markdown a').parentElement.previousElementSibling.textContent"),'Message\nsecond line');
      assert.equal(await browser.eval("getComputedStyle(document.querySelector('#activity .room-markdown p')).whiteSpace"),'pre-wrap');
      assert.equal(await browser.eval("!!document.querySelector('#activity img, #activity .event.result strong, #stdout strong')"),false);
      assert.equal(await browser.eval("document.getElementById('stdout').textContent"),raw);
      assert.match(await browser.eval("document.querySelector('#activity .event.result .event-text').textContent"),/literal \*\*raw\*\*/);
      await browser.eval("document.querySelector('[data-tab=logs]').click()");
      await until("document.querySelector('#stdout strong')?.textContent==='raw'");
      assert.equal(await browser.eval("document.querySelector('#stdout p').textContent"),'literal raw\nnext');
      await browser.eval("document.querySelector('[data-tab=activity]').click()");
      await browser.eval("document.querySelector('#activity .room-markdown a').click()");
      await until("SIM.log.some(x=>x.kind==='open_chat_link'&&x.p.url==='https://example.test/docs')");
      await browser.eval(`SIM.reply.evidence=()=>({report:SIM.room.report,test_output:[{task:'task-1',exit_code:0,stdout:${JSON.stringify(raw)},stderr:''}],diff:'+literal **diff**',truncated:false});document.querySelector('[data-tab="review"]').click();`);
      await until("!!document.querySelector('#test-results pre')&&!!document.querySelector('#diff .diff-line')");
      assert.equal(await browser.eval("document.querySelector('#test-results pre').textContent"),raw+'\n');
      assert.equal(await browser.eval("document.querySelector('#diff .diff-line').textContent"),'+literal **diff**');
      await browser.eval(`acResultView.show({goal:'Goal',summary:${JSON.stringify(samples.plan)},details:[{step_lessons:[{response:${JSON.stringify(samples.lesson)}}]}]});`);
      await until("!!document.querySelector('#result-summary h1')&&!!document.querySelector('#result-lessons strong')");

    }
    console.log('Room plan preview and history, activity, lessons, results and literal logs/diffs passed');
  } finally {
    if (browser) await browser.close();
    fs.rmSync(temp,{recursive:true,force:true});
  }
})().catch(error=>{console.error(error);process.exitCode=1;});
