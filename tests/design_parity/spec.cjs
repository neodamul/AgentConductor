// GitHub #414: one entry per element of the approved design (docs/design/approved-goal-ui). Each issue that
// builds an element adds its entry; `props` are computed-style properties that must match the mock exactly,
// width/height/mark within `tol` px, and the element's screenshot crop may differ by at most `maxPixelDiff`.
const BOX = ['borderTopWidth', 'borderTopLeftRadius'];
// Include the badge outside the avatar border; wrapping the real node preserves its actual pseudo-element.
const attentionBox = selector => `(() => { const e=document.querySelector(${JSON.stringify(selector)}),box=document.createElement('span');box.id='attention-probe';box.style.cssText='display:inline-flex;flex:none;padding:6px;background:transparent';e.replaceWith(box);box.append(e); })()`;
// The mock has no disabled step; its tabs are all reachable, so the app is compared with a run available (disabled steps dim by design).
const STEP_ONE = "document.querySelector('#start-step-tabs [role=tab]').click();";
const INSPECTOR_OPEN = "(async()=>{document.querySelectorAll('#start-step-tabs [role=tab]')[3].click();await new Promise(r=>setTimeout(r,900));document.querySelector('.inspector-switch').click();})()";
const STEPS_ENABLED = "SIM.home.run_available=true;SIM.home.run_state='running';";
// The reference window has a decorative border and outer margin; compare equal inner widths.
const STEP_REFERENCE_WIDTH = "document.querySelector('.win').style.cssText='width:calc(100vw + 2px);max-width:none'";
module.exports = [
  ...require('./typography.cjs'),
  ...require('./choice_spec.cjs'),
  { name: 'agent-avatar', props: BOX, // the mock's first agent waits for a decision: yellow border and the attention marker
   
    mock: { url: '?theme=light#0', selector: '.status-icon', index: 0, mark: '.agent-brand-logo' },
    app: { run: "acAgentStatus.apply([{ status: 'needs_input' }], [{ agent: 'claude' }], () => 'wait'); document.querySelectorAll('.agent-symbol').forEach(e => delete e.dataset.attention)", selector: '.agent-symbol.app-icon', index: 0, mark: 'content' } },
  { name: 'agent-attention-marker', props: [],
    mock: { url: '?theme=light#4', selector: '#attention-probe', run: "document.getElementById('rail-agents-tab').click();" + attentionBox('.agent .status-icon') },
    app: { selector: '#attention-probe', run: "acAgentStatus.apply([{status:'blocked_on_human'}],[{agent:'claude'}],()=> 'wait');" + attentionBox('.startup-profile .agent-symbol.app-icon') } },
  // The task rail of the run steps: the Room opens through the simulator's run and the Tasks tab.
  { name: 'task-avatar', props: BOX,
    mock: { url: '?theme=light#3', selector: '.outline-task .outline-agent', index: 0, mark: '.agent-brand-logo' },
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.plan.tasks[0].agent='claude';", run: "document.getElementById('start-tab-tasks').click()",
      selector: '.outline-task .app-icon.session-symbol', index: 0, mark: 'content' } },
  // Shell geometry (GitHub #408). Positions differ because the mock sits in a framed window, so x/y are not compared.
  { name: 'rail', props: [], keys: ['width'], pixel: false,
    mock: { selector: '.rail' }, app: { selector: '.startup-team' } },
  { name: 'rail-tab', props: ['borderTopLeftRadius', 'fontSize'], keys: ['width', 'height', 'borderTopLeftRadius', 'fontSize'], pixel: false,
    mock: { selector: '.rail-tabs button', index: 1 }, app: { selector: '#rail-tabs [role=tab]', index: 1 } },
  { name: 'step-tab', props: ['borderTopLeftRadius'], keys: ['height', 'borderTopLeftRadius'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.tabs .tab', index: 1 }, app: { selector: '#start-step-tabs [role=tab]', index: 1 } },
  { name: 'app-version', props: ['fontSize'], keys: ['fontSize'], pixel: false,
    mock: { selector: '.titlebar .ver' }, app: { selector: '.titlebar .app-version' } },
  { name: 'titlebar', props: [], keys: ['height'], pixel: false,
    mock: { selector: '.titlebar' }, app: { selector: 'header.titlebar' } },
  // Goal list row actions (GitHub #402): the trash is a 26px hit box with an 18px glyph, the folder 30px.
  { name: 'goal-trash', props: ['borderTopLeftRadius', 'color'], keys: ['width', 'height', 'borderTopLeftRadius', 'color'], pixel: false,
    mock: { url: '?theme=light#workspace', selector: '.goal-list-row>.goal-trash' },
    app: { setup: "SIM.home.slugs=[{slug:'refresh-token-rotation',goal:'g',active:true,progress:{stage:'completed',phases:[]}},{slug:'cache-cleanup',goal:'c',active:false,progress:{stage:'phase',phases:[]}}];", run: "document.getElementById('start-goal-list-open').click()", selector: '#start-goal-list .goal-trash' } },
  { name: 'goal-folder', props: ['borderTopLeftRadius'], keys: ['width', 'height', 'borderTopLeftRadius'], pixel: false,
    mock: { url: '?theme=light#workspace', selector: '.goal-list-row .goal-folder:not(.goal-trash)' },
    app: { setup: "SIM.home.slugs=[{slug:'refresh-token-rotation',goal:'g',active:true,progress:{stage:'completed',phases:[]}},{slug:'cache-cleanup',goal:'c',active:false,progress:{stage:'phase',phases:[]}}];", run: "document.getElementById('start-goal-list-open').click()", selector: '#start-goal-list .goal-folder' } },
  // Agent cards (GitHub #410): 8px radius, 22px tool buttons with 5px radius, the removal is the red 23px icon button.
  { name: 'agent-card', props: ['borderTopLeftRadius'], keys: ['width', 'borderTopLeftRadius'], pixel: false,
    mock: { selector: '.agent' }, app: { selector: '.startup-profile' } },
  { name: 'agent-tool', props: ['borderTopLeftRadius'], keys: ['height', 'borderTopLeftRadius'], pixel: false,
    mock: { selector: '.agent .tools button', index: 0 }, app: { selector: '.profile-tools .small-button', index: 0 } },
  { name: 'agent-remove', props: ['borderTopLeftRadius', 'color'], keys: ['width', 'height', 'borderTopLeftRadius', 'color'], pixel: false,
    mock: { selector: '.agent .tools button', index: 1 }, app: { selector: '.profile-tools .remove-agent', index: 0 } },
  // Task rows (GitHub #410): 4px radius, a 17px status tag; the chevron is a pseudo-element (checked in room_task_rows.rs).
  { name: 'task-row', props: ['borderTopLeftRadius'], keys: ['width', 'borderTopLeftRadius'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.outline-task' },
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '.session' } },
  { name: 'task-status-tag', props: ['borderTopLeftRadius'], keys: ['height', 'borderTopLeftRadius'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.outline-task .outline-state-badge' },
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '.outline-task .outline-state-badge' } },
  // Step groups (GitHub #408): a subdued background per Prepare / Run / Finish set with a 14px group icon.
  ...['prepare', 'run', 'finish'].map((group, index) => ({ name: `step-group-${group}`, props: ['borderTopLeftRadius', 'backgroundColor'], keys: ['width', 'height', 'borderTopLeftRadius', 'backgroundColor'],
    mock: { url: '?theme=light#0', run: STEP_REFERENCE_WIDTH, selector: '.tabs .step-group', index }, app: { setup: STEPS_ENABLED, run: STEP_ONE, selector: `#start-step-tabs .step-group[data-group=${group}]` } })),
  // #473: every tab is as wide as its caption and sits where the mock puts it.
  ...[0, 1, 2, 3, 4, 5].flatMap(index => [
    { name: `step-tab-box-${index}`, props: [], keys: ['width', 'height'],
      mock: { url: '?theme=light#0', run: STEP_REFERENCE_WIDTH, selector: '.tabs .tab', index }, app: { setup: STEPS_ENABLED, run: STEP_ONE, selector: '#start-step-tabs [role=tab]', index } },
    { name: `pos-step-tab-${index}`, props: [], keys: ['dt', 'dl'], pixel: false,
      mock: { url: '?theme=light#0', run: STEP_REFERENCE_WIDTH, selector: '.tabs .tab', index, rel: '.titlebar' }, app: { selector: '#start-step-tabs [role=tab]', index, rel: 'header.titlebar' } }]),
  // #468: the About dialog, opened by update (centered, with the update tag). Captured like every other box (lifted to a fixed
  // spot, so raster phase is identical on both sides); the mock re-centres open dialogs itself, so its observers are detached.
  { name: 'about-dialog', clone: true, flat: true, props: ['boxShadow', 'borderRadius'], keys: ['width', 'height', 'boxShadow', 'borderRadius'],
    mock: { url: '?theme=light&about=update#0', setup: 'referenceDialogResize.disconnect();referenceDialogOpen.disconnect();', selector: '#about-dialog' },
    app: { setup: "SIM.reply.about_status=()=>({version:'0.2.0',show:true,updated:true});", selector: '#about-dialog' } },
  { name: 'step-number-box', props: [], keys: ['width', 'height'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.step-group .tab .num', index: 0 }, app: { selector: '#start-step-tabs .tab-number', index: 0 } },
  { name: 'step-group-icon', props: [], keys: ['width', 'height'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.tabs-group-name svg', index: 0 }, app: { selector: '#start-step-tabs .tabs-group-name svg', index: 0 } },
  { name: 'step-group-label', props: ['fontSize', 'fontWeight', 'color'], keys: ['width', 'height', 'fontSize', 'fontWeight', 'color'],
    mock: { url: '?theme=light#0', selector: '.tabs-group-name .group-label', index: 0 }, app: { selector: '#start-step-tabs .tabs-group-name .group-label', index: 0 } },
  // #562: the footer's status dot and the Workspace folder / branch on its right match the mock's `.status`.
  { name: 'status-dot', props: ['backgroundColor', 'borderTopLeftRadius'], keys: ['width', 'height', 'backgroundColor', 'borderTopLeftRadius'],
    mock: { url: '?theme=light#0', selector: '.status .dot' }, app: { selector: '#startup-screen footer.statusbar .dot' } },
  { name: 'status-repository', clone: true, props: ['gap'], keys: ['width', 'height', 'gap'],
    mock: { url: '?theme=light#0', selector: '.status .status-repository' }, app: { selector: '#startup-screen footer.statusbar .status-repository' } },
  ...['worktree', 'branch'].map(part => ({ name: `status-${part}`, clone: true, props: ['fontSize', 'color'], keys: ['width', 'height', 'fontSize', 'color'],
    mock: { url: '?theme=light#0', selector: `.status .status-${part}` }, app: { selector: `#startup-screen footer.statusbar .status-${part}` } })),
  // #561: the right task-details panel opened on step 4 matches the mock's `#task-inspector-panel`.
  ...[
    { name: 'inspector-panel', props: ['backgroundColor', 'paddingTop', 'paddingLeft', 'borderLeftWidth'], keys: ['width', 'backgroundColor', 'paddingTop', 'paddingLeft', 'borderLeftWidth'], pixel: false, mock: '#task-inspector-panel', app: '.inspector' },
    { name: 'inspector-heading', props: ['fontSize', 'fontWeight', 'color'], keys: ['height', 'fontSize', 'fontWeight', 'color'], pixel: false, mock: '#task-inspector-title', app: '.inspector-heading h2' },
    { name: 'inspector-heading-box', props: ['paddingBottom', 'marginBottom', 'borderBottomWidth', 'borderBottomColor'], keys: ['height', 'paddingBottom', 'marginBottom', 'borderBottomWidth', 'borderBottomColor'], pixel: false, mock: '.task-inspector>.panel-heading', app: '.inspector-heading' },
    { name: 'inspector-section-title', props: ['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingBottom', 'borderBottomWidth'], keys: ['height', 'fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingBottom', 'borderBottomWidth'], pixel: false, mock: '.task-inspector .task-detail-content>section>h3', app: '.inspector section .eyebrow' },
    { name: 'inspector-section', props: ['paddingBottom', 'marginBottom', 'borderBottomWidth'], keys: ['paddingBottom', 'marginBottom', 'borderBottomWidth'], pixel: false, mock: '.task-inspector .task-detail-content>section', app: '.inspector section' },
    { name: 'inspector-instruction', props: ['fontSize', 'lineHeight', 'marginTop'], keys: ['fontSize', 'lineHeight', 'marginTop'], pixel: false, mock: '.task-inspector [data-detail-field="instruction"]', app: '#task-instruction' },
    { name: 'inspector-facts', props: ['gridTemplateColumns', 'rowGap', 'fontSize'], keys: ['gridTemplateColumns', 'rowGap', 'fontSize'], pixel: false, mock: '.task-inspector dl', app: '.task-facts' },
    { name: 'inspector-fact-label', props: ['color', 'fontSize'], keys: ['color', 'fontSize'], pixel: false, mock: '.task-inspector dt', app: '.task-facts dt' },
    { name: 'inspector-context', props: ['fontSize', 'lineHeight', 'color'], keys: ['height', 'fontSize', 'lineHeight', 'color'], pixel: false, mock: '#task-inspector-context', app: '#inspector-context' },
  ].map(({ mock, app, ...rest }) => ({ ...rest,
    mock: { url: '?theme=light#3', run: "document.getElementById('right-panel-toggle').click()", selector: mock },
    app: { setup: STEPS_ENABLED, run: INSPECTOR_OPEN, selector: app } })),
  // #563: dialogs of the approved design (mock id -> app id) share the mock's box and title.
  ...[['delete-goal-dialog', 'start-delete-goal', 'delete-goal'], ['close-run-dialog', 'start-close-dialog', 'close-run'], ['goal-result-dialog', 'result-dialog', 'goal-result'], ['profile-edit-dialog', 'start-agent-dialog', 'profile-edit']].flatMap(([mock, app, name]) => {
    const open = id => `document.getElementById('${id}').showModal()`, openApp = id => `document.getElementById('run-screen').hidden = false; ${open(id)}`;
    const mockSide = (selector) => ({ url: '?theme=light#0', setup: 'referenceDialogResize.disconnect();referenceDialogOpen.disconnect();', run: open(mock), selector });
    return [
      { name: `dialog-${name}`, props: ['paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor'], keys: ['width', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor'], pixel: false,
        mock: mockSide(`#${mock}`), app: { run: openApp(app), selector: `#${app}` } },
      ...(name === 'goal-result' ? [] : [{ name: `dialog-${name}-title`, props: ['fontSize', 'fontWeight', 'color'], keys: ['fontSize', 'fontWeight', 'color'], pixel: false,
        mock: mockSide(`#${mock} h2`), app: { run: openApp(app), selector: `#${app} h2` } }]),
    ];
  }),
  // #563: the Roles and Plan views' card chrome matches the mock's card of that view (hash #N = mock view, tab = app step).
  ...[[1, 'i', 1, 'roles'], [2, 'a', 2, 'plan']].flatMap(([hash, show, tab, name]) => [
    { name: `view-card-${name}`, props: ['paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'boxShadow'], keys: ['width', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'boxShadow'], pixel: false,
      mock: { url: `?theme=light#${hash}`, selector: `.main>.card[data-show="${show}"]` }, app: { setup: STEPS_ENABLED, run: `document.querySelectorAll('#start-step-tabs [role=tab]')[${tab}].click()`, selector: '.step-card:not([hidden])' } },
    { name: `view-title-${name}`, props: ['fontSize', 'fontWeight', 'color'], keys: ['height', 'fontSize', 'fontWeight', 'color'], pixel: false,
      mock: { url: `?theme=light#${hash}`, selector: `.main>.card[data-show="${show}"] .cardhead h2` }, app: { setup: STEPS_ENABLED, run: `document.querySelectorAll('#start-step-tabs [role=tab]')[${tab}].click()`, selector: '.step-card:not([hidden]) .step-title' } },
    { name: `view-eyebrow-${name}`, props: ['fontSize', 'fontWeight', 'color'], keys: ['height', 'fontSize', 'fontWeight', 'color'], pixel: false,
      mock: { url: `?theme=light#${hash}`, selector: `.main>.card[data-show="${show}"] .cardhead .eyebrow` }, app: { setup: STEPS_ENABLED, run: `document.querySelectorAll('#start-step-tabs [role=tab]')[${tab}].click()`, selector: '.step-card:not([hidden]) .eyebrow' } },
  ]),
  // #580: the Room's task view is one step card like the mock's (hash #3 = view "b", tab 3 = Tasks).
  ...[['card', '', '.room-card', ['paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor'], ['paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor']],
    ['eyebrow', ' .cardhead .eyebrow', '.room-card .eyebrow', ['fontSize', 'fontWeight', 'color'], ['height', 'fontSize', 'fontWeight', 'color']],
    ['title', ' .cardhead h2', '.room-card #selected-agent', ['fontSize', 'fontWeight', 'color'], ['fontSize', 'fontWeight', 'color']],
    ['meta', ' .cardhead .sub', '.room-card #selected-task', ['fontSize', 'color'], ['fontSize', 'color']],
  ].map(([name, mock, app, props, keys]) => ({ name: `room-card-${name}`, props, keys, pixel: false,
    mock: { url: '?theme=light#3', selector: `.main>.card[data-show="b"]${mock}` }, app: { setup: STEPS_ENABLED, run: "document.querySelectorAll('#start-step-tabs [role=tab]')[3].click()", selector: app } })),
  // #597: the Room's verify card (mock view #6 = "f"): coordinator view in the verify stage.
  ...(() => {
    // GitHub #689: an approval-waiting run shows the summary card; the verify rows belong to the run that is still verifying.
    const setup = STEPS_ENABLED + "SIM.room.report.state='running';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.tests=[{task:'task-1',exit_code:0,timed_out:false,output:''}];";
    const run = "document.querySelectorAll('#start-step-tabs [role=tab]')[4].click(); document.querySelector('.final-step .session-choice').click()";
    const card = '.main>.card[data-show="f"]';
    return [['eyebrow', ' .cardhead .eyebrow', '.room-card .eyebrow', ['fontSize', 'fontWeight', 'color'], ['height', 'fontSize', 'fontWeight', 'color']],
      ['title', ' .cardhead h2', '.room-card #selected-agent', ['fontSize', 'fontWeight', 'color'], ['fontSize', 'fontWeight', 'color']],
      ['row', ' .row', '.verify-row', ['borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize'], ['height', 'borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize']],
      ['row-title', ' .row .t', '.verify-row .t', ['fontSize', 'fontWeight'], ['fontSize', 'fontWeight']],
      ['row-glyph', ' .row .n', '.verify-row .n', ['borderTopLeftRadius', 'backgroundColor', 'fontSize'], ['width', 'height', 'borderTopLeftRadius', 'backgroundColor', 'fontSize']],
    ].map(([name, mock, app, props, keys]) => ({ name: `verify-card-${name}`, props, keys, pixel: false,
      mock: { url: '?theme=light#6', selector: card + mock }, app: { setup, run, selector: app } }));
  })(),
  // #598: the Room's review card (mock view #7 = "c"): coordinator view once the run waits for approval.
  ...(() => {
    const setup = STEPS_ENABLED + "SIM.room.report.state='waiting_approval';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'abcdef1234567890'};SIM.room.report.tests=[{task:'task-1',exit_code:0,timed_out:false,output:''}];SIM.reply.evidence=()=>({revision:'x1',report:SIM.room.report,truncated:false,test_output:[{task:'task-1',exit_code:0,timed_out:false,stdout:'',stderr:''}],diff:'diff --git a/src/auth/refresh.rs b/src/auth/refresh.rs\\n--- a/src/auth/refresh.rs\\n+++ b/src/auth/refresh.rs\\n@@ -1 +1 @@\\n-a\\n+b\\n+c\\ndiff --git a/docs/security.md b/docs/security.md\\n+++ b/docs/security.md\\n+x\\n'});";
    const run = "document.querySelectorAll('#start-step-tabs [role=tab]')[5].click(); document.querySelector('.final-step .session-choice').click()";
    const card = '.main>.card[data-show="c"]';
    return [['eyebrow', ' .cardhead .eyebrow', '.room-card .eyebrow', ['fontSize', 'fontWeight', 'color'], ['height', 'fontSize', 'fontWeight', 'color']],
      ['title', ' .cardhead h2', '.room-card #selected-agent', ['fontSize', 'fontWeight', 'color'], ['fontSize', 'fontWeight', 'color']],
      ['stat', ' .stat', '.review-summary .stat', ['paddingTop', 'borderTopWidth', 'lineHeight'], ['height', 'paddingTop', 'borderTopWidth', 'lineHeight']],
      ['stat-label', ' .stat .l', '.review-summary .l', ['fontSize', 'fontWeight', 'color'], ['fontSize', 'fontWeight', 'color']],
      ['stat-value', ' .stat .v', '.review-summary .v', ['fontSize', 'fontWeight', 'color'], ['fontSize', 'fontWeight', 'color']],
      ['files', ' .files', '.review-summary .files', ['borderTopLeftRadius', 'fontSize'], ['borderTopLeftRadius', 'fontSize']],
      ['file-row', ' .files>div', '.review-summary .files>div', ['paddingTop', 'paddingLeft'], ['height', 'paddingTop', 'paddingLeft']],
      ['file-add', ' .files .add', '.review-summary .add', ['color', 'backgroundColor', 'borderTopLeftRadius', 'paddingTop', 'fontSize'], ['height', 'color', 'backgroundColor', 'borderTopLeftRadius', 'paddingTop', 'fontSize']],
    ].map(([name, mock, app, props, keys]) => ({ name: `review-card-${name}`, props, keys, pixel: false,
      mock: { url: '?theme=light#7', selector: card + mock }, app: { setup, run, selector: app } }));
  })(),
  // GitHub #689: the approval footer and the heading line under the title (mock view #7 = "c").
  ...(() => {
    const setup = STEPS_ENABLED + "SIM.room.report.state='waiting_approval';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'abcdef1234567890'};SIM.room.report.tests=[{task:'task-1',exit_code:0,timed_out:false,output:''}];SIM.reply.evidence=()=>({revision:'x1',report:SIM.room.report,truncated:false,test_output:[],diff:''});";
    const run = "document.querySelectorAll('#start-step-tabs [role=tab]')[5].click(); document.querySelector('.final-step .session-choice').click()";
    const card = '.main>.card[data-show="c"]';
    return [['note', ' .actions .note', '#review-footer .note', ['fontSize', 'color'], ['fontSize', 'color']],
      ['deny', ' .actions .btn.q', '#review-deny', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius']],
      ['primary', ' .actions .btn.p', '#review-primary', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color']],
    ].map(([name, mock, app, props, keys]) => ({ name: `review-footer-${name}`, props, keys, pixel: false,
      mock: { url: '?theme=light#7', selector: card + mock }, app: { setup, run, selector: app } }))
      .concat([{ name: 'review-heading-info', props: ['fontSize', 'color'], keys: ['height', 'fontSize', 'color'], pixel: false,
        mock: { url: '?theme=light#7', selector: '.runmeta[data-show="c"] span' }, app: { setup, run, selector: '#run-goal-info .goal-info-content' } },
      { name: 'review-card-has-no-tabs', assert: "(() => { const c = document.querySelector('#run-screen .room-card'); return c.classList.contains('review-only') && getComputedStyle(c.querySelector('.view-tabs')).display === 'none' && !document.getElementById('review-footer').hidden; })()", app: { setup, run } }]);
  })(),
  // #599: the task card's stop reason (mock views #5 "g" = tool request wait, #6 "h" = permission denied).
  ...[['wait', 4, 'g', 'warn-bg', ".task-notice.warn", "SIM.room.tool_requests=[{id:'tr1',status:'pending',context:{task:'task-1'},request:{tool:'Bash',input:{command:'npm install jsonwebtoken@9'}}}];"],
    ['denied', 5, 'h', 'err-bg', '.task-notice.err', "SIM.room.report.tasks[0].status='permission_denied';SIM.room.report.tasks[0].error='permission denied: tool bash';"]].flatMap(([name, hash, show, bg, app, preset]) => {
    const mock = `.main>.card[data-show="${show}"] div[style*="${bg}"]`, setup = STEPS_ENABLED + preset, run = "document.querySelectorAll('#start-step-tabs [role=tab]')[3].click()";
    return [{ name: `task-notice-${name}`, props: ['paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color', 'fontSize'], keys: ['paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color', 'fontSize'], pixel: false,
      mock: { url: `?theme=light#${hash}`, selector: mock }, app: { setup, run, selector: app } },
    { name: `task-notice-${name}-title`, props: ['fontWeight', 'fontSize'], keys: ['fontWeight', 'fontSize'], pixel: false,
      mock: { url: `?theme=light#${hash}`, selector: mock + ' b' }, app: { setup, run, selector: app + ' b' } }];
  }),
  // #605: the choice popup (mock #assignment-menu = app .ac-choice-menu) shares the mock's box, heading and options.
  ...[['box', '#assignment-menu', '.ac-choice-menu:not([hidden])', ['paddingTop', 'borderTopLeftRadius', 'backgroundColor', 'boxShadow', 'fontSize'], ['width', 'paddingTop', 'borderTopLeftRadius', 'backgroundColor', 'boxShadow', 'fontSize']],
    ['heading', '#assignment-menu-heading', '.ac-choice-heading', ['paddingTop', 'paddingLeft', 'fontSize', 'color'], ['height', 'paddingTop', 'paddingLeft', 'fontSize', 'color']],
    ['option', '#assignment-options>button', '.ac-choice-options>button', ['paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'fontSize', 'color'], ['height', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'fontSize', 'color']],
  ].map(([name, mock, app, props, keys]) => ({ name: `assignment-menu-${name}`, props, keys, pixel: false,
    mock: { url: '?theme=light#1', run: "document.querySelector('.assignment-choice').click()", selector: mock },
    app: { run: "document.querySelector('#start-panel-workspace .ac-choice-trigger').click()", selector: app, index: name === 'option' ? 1 : 0 } })),
  // #607: activity rows (mock `.ev.th` = app `.event.muted`, the thinking row): number column, title, italic detail.
  ...(() => {
    const setup = STEPS_ENABLED + "SIM.reply.task=()=>({task:{id:'task-1'},events:[{seq:3,payload:{type:'thinking_tokens'}},{seq:14,payload:{type:'thinking_tokens'}}],decisions:[],coordinator_decisions:[],knowledge:[],step_lessons:[],usage:null,stdout:'',stderr:''});";
    const run = "document.querySelectorAll('#start-step-tabs [role=tab]')[3].click()", card = '.main>.card[data-show="b"] .ev.th';
    return [['row', '', '#activity .event.muted', ['paddingTop', 'paddingBottom', 'borderBottomWidth', 'fontSize']],
      ['number', ' code', '#activity .event.muted .event-seq', ['fontSize', 'color']],
      ['title', ' b', '#activity .event.muted .event-head', ['fontSize', 'fontWeight', 'color']],
      ['detail', ' .m', '#activity .event.muted .event-text', ['fontSize', 'fontStyle', 'color']],
    ].map(([name, mock, app, props]) => ({ name: `activity-${name}`, props, keys: props, pixel: false,
      mock: { url: '?theme=light#3', selector: card + mock }, app: { setup, run, selector: app } }));
  })(),
  // #563: the slide-over panels of the design (mock id -> app dialog) share the mock's box and heading.
  ...[['settings-panel', 'start-settings', 'settings'], ['coordinator-chat-panel', 'start-chat', 'chat'], ['workspace-overview', 'start-goals', 'goals']].flatMap(([mock, app, name]) => [
    { name: `panel-${name}`, props: ['paddingTop', 'paddingLeft', 'backgroundColor', 'borderLeftWidth'], keys: ['width', 'paddingTop', 'paddingLeft', 'backgroundColor', 'borderLeftWidth'], pixel: false,
      mock: { url: '?theme=light#0', run: `openSidePanel('${mock}')`, selector: `#${mock}` }, app: { run: `document.getElementById('run-screen').hidden = false; const el = document.getElementById('${app}'); el.showModal ? el.showModal() : (el.hidden = false)`, selector: `#${app}` } },
  ]),
  // #563: the application menu popup (behavior.md: the mock's short menu is illustrative, so items differ; the box and rows match).
  ...[
    { name: 'app-menu-box', props: ['borderTopLeftRadius', 'paddingTop', 'borderTopWidth', 'backgroundColor'], keys: ['borderTopLeftRadius', 'paddingTop', 'borderTopWidth', 'backgroundColor'], mock: '#app-menu', app: '#start-workspace-list' },
    { name: 'app-menu-item', props: ['borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize', 'color'], keys: ['height', 'borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize', 'color'], mock: '#app-menu button', app: '#start-workspace-list button' },
  ].map(({ mock, app, ...rest }) => ({ ...rest, pixel: false,
    mock: { url: '?theme=light#0', run: "document.getElementById('app-menu-button').click()", selector: mock },
    app: { run: "(async () => { document.getElementById('start-app-menu').click(); await new Promise(r => setTimeout(r, 900)); })()", selector: app } })),
  // #579: every step card's footer buttons share one size (34px, 8px/13px padding, 6px radius, 13px text).
  ...[[0, 'j', 0, 'workspace'], [1, 'i', 1, 'roles'], [2, 'a', 2, 'plan']].flatMap(([hash, show, tab, name]) => [['p', 'primary'], ['q', 'quiet']].map(([kind, appClass]) => (
    { name: `footer-button-${kind}-${name}`, props: ['paddingTop', 'paddingLeft', 'fontSize', 'borderTopLeftRadius'], keys: ['height', 'paddingTop', 'paddingLeft', 'fontSize', 'borderTopLeftRadius'], pixel: false,
      mock: { url: `?theme=light#${hash}`, selector: `.main>.card[data-show~="${show}"] .actions .btn.${kind}` },
      app: { setup: STEPS_ENABLED, run: `document.querySelectorAll('#start-step-tabs [role=tab]')[${tab}].click()`, selector: `.step-card:not([hidden]) :is(.startup-actions, .startup-review-footer) button.${appClass}`, index: 0 } }))),
  ...[[0, 'workspace'], [1, 'roles'], [2, 'plan']].map(([tab, name]) => ({ name: `footer-buttons-one-size-${name}`,
    assert: `(() => { const b = [...document.querySelectorAll('.step-card:not([hidden]) :is(.startup-actions, .startup-review-footer) button')].filter(x => x.getBoundingClientRect().width > 0); const h = b.map(x => x.getBoundingClientRect().height); return b.length >= 2 && Math.max(...h) - Math.min(...h) <= 1.5 && h.every(v => Math.abs(v - 34) <= 2); })()`,
    app: { setup: STEPS_ENABLED, run: `document.querySelectorAll('#start-step-tabs [role=tab]')[${tab}].click()` } })),
  // #583: the Workspace footer's idle text is the design's "connected - local run" (the folder is named on the right).
  // #600 (mock view #10): after "New Goal" the head state and the footer note name the new Goal.
  { name: 'new-goal-state', assert: "(() => document.getElementById('start-current-goal-state').textContent.trim() === '새 목표' && document.getElementById('start-save-note').textContent.trim() === '작업 디렉터리·코디네이터는 유지하고 새 Goal을 추가합니다.')()", app: { setup: "SIM.home.config.goal='';SIM.home.slugs=[{slug:'a',goal:'g',active:false,progress:{}}];" } },
  { name: 'new-goal-state-default', assert: "(() => document.getElementById('start-save-note').textContent.trim() === 'Workspace 설정과 Goal을 저장한 뒤 역할을 확인합니다.')()", app: { setup: "SIM.home.slugs=[{slug:'a',goal:'g',active:true,progress:{}}];" } },
  { name: 'status-text', assert: "(() => document.getElementById('start-status').textContent.trim() === '연결됨 · 로컬 실행')()", app: { setup: STEPS_ENABLED } },
  // Step card (GitHub #408): the header and the action footer stay visible while only the body scrolls.
  { name: 'step-card-fixed-header-footer', viewport: {height:520},
    app: { run: "const m = document.querySelector('.startup-main'); m.scrollTop = Math.floor(m.scrollHeight / 2)" },
    assert: "(() => { const c = document.querySelector('#start-panel-workspace'), h = c.querySelector('.cardhead').getBoundingClientRect(), f = c.querySelector('.startup-actions').getBoundingClientRect(); return h.top >= 0 && h.bottom <= innerHeight && f.top >= 0 && f.bottom <= innerHeight + 1; })()" },
  // Content-width inner tabs (GitHub #408): 29px high, 7px radius, label + 11px padding, left aligned, not stretched.
  { name: 'view-tab', props: ['borderTopLeftRadius', 'fontSize'], keys: ['height', 'borderTopLeftRadius', 'fontSize'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.ttabs span', index: 1 },
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '.view-tabs .tab', index: 1 } },
  { name: 'view-tabs-not-stretched', assert: "(() => { const t = [...document.querySelectorAll('.view-tabs .tab')]; const w = document.querySelector('.view-tabs').getBoundingClientRect().width; return t.length >= 4 && t.every(x => x.getBoundingClientRect().width < w / 3); })()",
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()" } },
  // Compact Goal header (GitHub #408): 17px semibold single line.
  { name: 'goal-title', props: ['fontSize', 'fontWeight'], keys: ['height', 'fontSize', 'fontWeight'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.head h1' }, app: { selector: '.startup-main .startup-heading h1' } },
  // Left rail toggle (GitHub #408): a 32px titlebar button hides and restores the whole rail; the centre takes the space.
  { name: 'rail-toggle-button', props: ['borderTopLeftRadius'], keys: ['width', 'height', 'borderTopLeftRadius'], pixel: false,
    mock: { url: '?theme=light#0', selector: '#sidebar-toggle' }, app: { selector: '#start-rail-toggle' } },
  { name: 'rail-toggle-behaviour',
    app: { run: "window.__w = document.querySelector('.startup-main').getBoundingClientRect().width; document.getElementById('start-rail-toggle').click()" },
    assert: "(() => { const hidden = document.querySelector('.startup-team').getBoundingClientRect().width === 0 && document.querySelector('.startup-main').getBoundingClientRect().width > window.__w + 200; document.getElementById('start-rail-toggle').click(); return hidden && document.querySelector('.startup-team').getBoundingClientRect().width === (innerWidth < 900 ? 230 : 250); })()" },
  // Rail width follows the design's breakpoints: 250px, 230px below 900px (GitHub #408).
  { name: 'rail-width-840', viewport: [840, 800], app: {}, assert: "document.querySelector('.startup-team').getBoundingClientRect().width === 230" },
  { name: 'rail-width-1280', viewport: [1280, 800], app: {}, assert: "document.querySelector('.startup-team').getBoundingClientRect().width === 250" },
  // Result summary (GitHub #408): compact metadata on one ruled line - 16px/650 values, 14px/500 labels.
  { name: 'result-metric-value', props: ['fontSize', 'fontWeight'], keys: ['fontSize', 'fontWeight'], pixel: false,
    mock: { url: '?theme=light#8', selector: '.result-stats strong' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n+1\\n',tests:[{task:'t',exit_code:0}],details:[]})", selector: '.result-metric strong' } },
  { name: 'result-metric-label', props: ['fontSize', 'fontWeight'], keys: ['fontSize', 'fontWeight'], pixel: false,
    mock: { url: '?theme=light#8', selector: '.result-stats span' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n+1\\n',tests:[{task:'t',exit_code:0}],details:[]})", selector: '.result-metric span' } },
  // Full-window behaviour (GitHub #408): a big window reflows the centre; rail, icons, tabs and borders keep their size.
  { name: 'fullscreen-reflow', viewport: [1920, 1080], app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()" },
    assert: "(() => { const r = n => document.querySelector(n).getBoundingClientRect(); const q = n => getComputedStyle(document.querySelector(n)); return r('.rail').width === 250 && r('.session-symbol').width === 28 && r('.view-tabs .tab').height === 29 && q('.session-symbol').borderTopWidth === '1.5px' && r('.center').width > 1200; })()" },
  // Slide-over panels (GitHub #406): the team briefing is 600px wide with 24px padding, anchored to the right edge.
  { name: 'briefing-panel', props: ['paddingTop', 'paddingLeft'], keys: ['width', 'paddingTop', 'paddingLeft'], pixel: false,
    mock: { url: '?theme=light&panel=briefing#0', selector: '.briefing-panel' },
    app: { run: "document.getElementById('run-screen').hidden = false; document.getElementById('briefing-dialog').showModal()", selector: '#briefing-dialog' } },
  // Titlebar cluster (GitHub #408): 32px hit boxes with a 6px radius and a transparent border at rest.
  { name: 'titlebar-chat', props: ['borderTopLeftRadius', 'borderTopColor'], keys: ['width', 'height', 'borderTopLeftRadius', 'borderTopColor'], pixel: false,
    mock: { selector: '#coordinator-chat-button' }, app: { selector: '#start-chat-button' } },
  { name: 'titlebar-settings', props: ['borderTopLeftRadius', 'borderTopColor'], keys: ['width', 'height', 'borderTopLeftRadius', 'borderTopColor'], pixel: false,
    mock: { selector: '#settings-button' }, app: { selector: '#start-settings-button' } },
  // GitHub #673: the update icon and the right-panel toggle are always in the titlebar (update, chat, settings, panel toggle).
  { name: 'titlebar-update', props: ['borderTopLeftRadius', 'borderTopColor'], keys: ['width', 'height', 'borderTopLeftRadius', 'borderTopColor'], pixel: false,
    mock: { selector: '#update-button' }, app: { selector: '#start-update-button' } },
  { name: 'titlebar-inspector-toggle', props: ['borderTopLeftRadius', 'borderTopColor'], keys: ['width', 'height', 'borderTopLeftRadius', 'borderTopColor'], pixel: false,
    mock: { selector: '#right-panel-toggle' }, app: { selector: '#start-inspector-toggle' } },
  // The docked task inspector is available before any run exists and starts closed, like the mock's.
  { name: 'startup-inspector-open', props: ['backgroundColor', 'paddingTop', 'paddingLeft', 'borderLeftWidth'], keys: ['width', 'backgroundColor', 'paddingTop', 'paddingLeft', 'borderLeftWidth'], pixel: false,
    mock: { url: '?theme=light#0', run: STEP_REFERENCE_WIDTH + ";document.getElementById('right-panel-toggle').click()", selector: '#task-inspector-panel' },
    app: { run: "document.getElementById('start-inspector-toggle').click()", selector: '#start-inspector' } },
  // Closed by default, and docked at the right edge of the window once opened (not in the rail or below the card).
  // GitHub #677: the titlebar holds only the mock's controls, on the setup steps and while a run is open.
  { name: 'titlebar-controls', assert: "(() => [...document.querySelectorAll('header.titlebar button')].filter(b => b.getClientRects().length).map(b => b.id).join() === 'start-app-menu,start-rail-toggle,start-workspace-menu,start-update-button,start-chat-button,start-settings-button,start-inspector-toggle')()",
    app: { setup: STEPS_ENABLED } },
  // GitHub #678: a completed step shows a check mark in place of its number.
  { name: 'step-done-number', props: ['color', 'backgroundColor', 'fontSize'], keys: ['width', 'height', 'color', 'backgroundColor'],
    mock: { url: '?theme=light#3', selector: '.tabs .tab.done .num', index: 0 },
    app: { setup: STEPS_ENABLED, run: "document.getElementById('start-tab-tasks').click()", selector: '#start-step-tabs [role=tab].done .tab-number', index: 0 } },
  // GitHub #675 / #676: the run screen's rail and top area against the mock's step 4.
  ...[
    ['run-rail-head', ['fontSize', 'fontWeight'], ['height', 'fontSize', 'fontWeight'], '#rail-tasks-panel .railhead h2', '#rail-room .railhead h2'],
    ['run-rail-current', ['fontSize', 'color'], ['height', 'fontSize', 'color'], '#outline-total', '#rail-current'],
    ['run-rail-status', ['fontSize', 'color', 'paddingTop', 'paddingBottom', 'borderBottomWidth'], ['height', 'fontSize', 'color', 'paddingTop', 'paddingBottom', 'borderBottomWidth'], '.outline-run-status', '#rail-room .outline-run-status'],
    ['run-rail-phase', ['fontSize', 'fontWeight', 'paddingTop', 'borderBottomWidth'], ['height', 'fontSize', 'fontWeight', 'paddingTop', 'borderBottomWidth'], '.phase-tree summary', '#rail-room .phase-tree summary'],
    ['run-rail-task', ['paddingTop', 'paddingLeft', 'borderBottomWidth', 'borderTopLeftRadius'], ['width', 'paddingTop', 'paddingLeft', 'borderBottomWidth', 'borderTopLeftRadius'], '.outline-task', '#rail-room .outline-task'],
    ['run-rail-task-title', ['fontSize', 'fontWeight'], ['fontSize', 'fontWeight'], '.outline-task strong', '#rail-room .outline-task strong'],
    ['run-rail-note', ['fontSize', 'color', 'paddingTop', 'borderTopWidth'], ['fontSize', 'color', 'paddingTop', 'borderTopWidth'], '.outline-note', '#rail-room .outline-note'],
    ['run-heading', ['fontSize', 'fontWeight'], ['height', 'fontSize', 'fontWeight'], '#saved-goal-title', '#run-screen .goal-bar h1'],
  ].map(([name, props, keys, mock, app]) => ({ name, props, keys, pixel: false,
    mock: { url: '?theme=light#3', selector: mock },
    app: { setup: STEPS_ENABLED, run: "document.getElementById('start-tab-tasks').click()", selector: app } })),
  // GitHub #685: the Agent roles step against the mock's step 2.
  ...[
    ['roles-header', ['fontSize', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth'], ['height', 'fontSize', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth'], '.role-choice-header', '#start-roles thead tr'],
    ['roles-row', ['paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopLeftRadius', 'marginTop'], ['height', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopLeftRadius', 'marginTop'], '.role-choice-row', '#start-roles tbody tr:not(.role-planning)'],
    ['roles-name', ['fontSize', 'fontWeight'], ['fontSize', 'fontWeight'], '.role-choice-row .t', '#start-roles tbody tr:not(.role-planning) td:first-child'],
    ['roles-title', ['fontSize', 'fontWeight'], ['height', 'fontSize', 'fontWeight'], '.main>.card[data-show="i"] h2, .card[data-show="i"] h2', '#start-roles-title'],
  ].map(([name, props, keys, mock, app]) => ({ name, props, keys, pixel: false,
    mock: { url: '?theme=light#1', selector: mock },
    app: { run: "document.getElementById('start-tab-team').click()", selector: app } })),
  // GitHub #686: the plan-review rail (mock step 3) is the same outline as the run rail.
  ...[
    ['plan-rail-head', ['fontSize', 'fontWeight'], ['height', 'fontSize', 'fontWeight'], '#rail-tasks-panel .railhead h2', '#rail-outline h2'],
    ['plan-rail-current', ['fontSize', 'color'], ['height', 'fontSize', 'color'], '#outline-total', '#rail-outline .sub'],
    ['plan-rail-status', ['fontSize', 'color', 'paddingTop', 'paddingBottom', 'borderBottomWidth'], ['height', 'fontSize', 'color', 'paddingTop', 'paddingBottom', 'borderBottomWidth'], '.outline-run-status', '#rail-outline .outline-run-status'],
    ['plan-rail-phase', ['fontSize', 'fontWeight', 'paddingTop', 'borderBottomWidth'], ['height', 'fontSize', 'fontWeight', 'paddingTop', 'borderBottomWidth'], '.phase-tree summary', '#rail-outline .phase-tree summary'],
    ['plan-rail-task', ['paddingTop', 'paddingLeft', 'borderBottomWidth', 'borderTopLeftRadius'], ['width', 'paddingTop', 'paddingLeft', 'borderBottomWidth', 'borderTopLeftRadius'], '.outline-task', '#rail-outline .outline-task'],
    ['plan-rail-note', ['fontSize', 'color', 'paddingTop', 'borderTopWidth'], ['fontSize', 'color', 'paddingTop', 'borderTopWidth'], '.outline-note', '#rail-outline .outline-note'],
  ].map(([name, props, keys, mock, app]) => ({ name, props, keys, pixel: false,
    mock: { url: '?theme=light#2', selector: mock },
    app: { setup: "SIM.home.proposal=SIM.proposal('ready');", run: "document.getElementById('start-tab-review').click()", selector: app } })),
  // GitHub #684: a pending tool request is decided in the card footer (mock step 5).
  ...[
    ['tool-decision-note', ['fontSize', 'color'], ['fontSize', 'color'], '.card[data-show="g"] .actions .note', '#tool-decision-note'],
    ['tool-decision-deny', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor'], '.card[data-show="g"] .btn.q', '#tool-decision-deny'],
    ['tool-decision-allow', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color'], '.card[data-show="g"] .btn.p', '#tool-decision-allow'],
  ].map(([name, props, keys, mock, app]) => ({ name, props, keys, pixel: false,
    mock: { url: '?theme=light#4', selector: mock },
    app: { setup: STEPS_ENABLED + "SIM.room.tool_requests=[{id:'tr1',status:'pending',created_at:Math.floor(Date.now()/1000),context:{task:'task-1'},request:{tool:'Bash',input:{command:'npm install jsonwebtoken@9'}}}];", run: "document.querySelectorAll('#start-step-tabs [role=tab]')[3].click()", selector: app } })),
  { name: 'tool-decision-replaces-direction', assert: "(() => !document.getElementById('tool-decision').hidden && document.querySelector('#run-screen .composer').hidden && !document.querySelector('dialog[open]'))()",
    app: { setup: STEPS_ENABLED + "SIM.room.tool_requests=[{id:'tr1',status:'pending',created_at:Math.floor(Date.now()/1000),context:{task:'task-1'},request:{tool:'Bash',input:{command:'npm install jsonwebtoken@9'}}}];", run: "document.querySelectorAll('#start-step-tabs [role=tab]')[3].click()" } },
  { name: 'startup-inspector-closed', assert: "document.getElementById('start-inspector').getBoundingClientRect().width===0", app: {} },
  { name: 'startup-inspector-docked', assert: "(()=>{const r=document.querySelector('#start-inspector').getBoundingClientRect();return Math.round(r.right)===innerWidth&&r.top<200&&r.height>300})()",
    app: { run: "document.getElementById('start-inspector-toggle').click()" } },
];
