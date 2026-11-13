// Extracted from spec.cjs to keep each comparison file under 300 lines.
const {BOX,attentionBox,STEP_ONE,INSPECTOR_OPEN,STEPS_ENABLED,STEP_REFERENCE_WIDTH}=require('./spec_context.cjs');
module.exports=[
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
  // GitHub #745: the selected and the unselected rail tab differ in fill and text colour.
  { name: 'rail-tab-selected', props: ['backgroundColor', 'color', 'fontWeight', 'borderTopColor'], keys: ['backgroundColor', 'color', 'fontWeight', 'borderTopColor'], pixel: false,
    mock: { selector: '.rail-tabs button[aria-selected="true"]' }, app: { selector: '#rail-tabs [role=tab][aria-selected=true]' } },
  { name: 'rail-tab-unselected', props: ['backgroundColor', 'color', 'fontWeight', 'borderTopColor'], keys: ['backgroundColor', 'color', 'fontWeight', 'borderTopColor'], pixel: false,
    mock: { selector: '.rail-tabs button[aria-selected="false"]' }, app: { selector: '#rail-tabs [role=tab][aria-selected=false]' } },
  // GitHub #748: the Goal list button, the team button and the status text sit 12px apart.
  { name: 'heading-gap', props: ['columnGap'], keys: ['columnGap', 'height'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.main>.head' }, app: { selector: '.startup-heading' } },
  { name: 'run-heading-gap', props: ['columnGap'], keys: ['columnGap', 'height'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.main>.head' }, app: { setup: STEPS_ENABLED, run: "document.getElementById('start-tab-tasks').click()", selector: '#run-screen .goal-bar' } },
  // GitHub #892: status tags share the agent tag border, corners and padding.
  { name: 'heading-pill', props: ['borderTopWidth', 'borderTopLeftRadius', 'paddingTop', 'paddingLeft'], keys: ['y', 'height', 'borderTopWidth', 'borderTopLeftRadius', 'paddingTop', 'paddingLeft'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.goal-info>.pill', rel: '.goal-info' }, app: { selector: '#start-goal-info>.status-pill', rel: '#start-goal-info' } },
  { name: 'run-heading-pill', props: [], keys: ['y', 'height'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.goal-info>.pill', rel: '.goal-info' }, app: { setup: STEPS_ENABLED, run: "document.getElementById('start-tab-tasks').click()", selector: '#run-goal-info>.status-pill', rel: '#run-goal-info' } },
  // GitHub #750: the agent dialog uses one label gap, one field gap and one single-line control height.
  ...[
    ['agent-dialog-id-input', ['height', 'borderTopLeftRadius'], '#profile-id', '#start-agent-id', null, null],
    ['agent-dialog-exe-input', ['height'], '#profile-executable', '#start-executable', null, null],
    ['agent-dialog-browse', ['height'], '#profile-browse', '#start-executable-browse', null, null],
    ['agent-dialog-quota-input', ['height'], '#profile-quota-minutes', '#start-quota-minutes', null, null],
    ['agent-dialog-first-field', ['y'], '#profile-id', '#start-agent-id', '#profile-edit-title', '#start-agent-title'],
    ['agent-dialog-row-gap', ['y'], '#profile-executable', '#start-executable', '#profile-id', '#start-agent-id'],
    ['agent-dialog-label-gap', ['y'], '#profile-id', '#start-agent-id', '#profile-edit-dialog .profile-editor-grid>label:nth-child(2)', '#start-agent-dialog .agent-form-grid>label:nth-child(2)'],
  ].map(([name, keys, mock, app, mockRel, appRel]) => ({ name, props: [], keys, pixel: false,
    mock: { url: '?theme=light#0', setup: 'referenceDialogResize.disconnect();referenceDialogOpen.disconnect();', run: "document.getElementById('profile-edit-dialog').showModal()", selector: mock, ...(mockRel ? { rel: mockRel } : {}) },
    app: { run: "document.getElementById('run-screen').hidden = false; document.getElementById('start-agent-dialog').showModal()", selector: app, ...(appRel ? { rel: appRel } : {}) } })),
  // GitHub #746: the body area shows the window surface behind the cards.
  { name: 'main-background', props: ['backgroundColor'], keys: ['backgroundColor'], pixel: false,
    mock: { selector: '.win' }, app: { selector: '.startup-main' } },
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
  { name: 'task-status-tag', props: ['borderTopLeftRadius', 'borderTopWidth', 'backgroundColor', 'color'], keys: ['height', 'borderTopLeftRadius', 'borderTopWidth', 'backgroundColor', 'color'], pixel: false,
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
    const run = "document.querySelectorAll('#start-step-tabs [role=tab]')[4].click(); (async () => { for (let i = 0; i < 200 && !document.querySelector('.final-step .session-choice'); i++) await new Promise(r => setTimeout(r, 50)); document.querySelector('.final-step .session-choice').click(); for (let i = 0; i < 200 && !document.querySelector('.verify-row, .review-summary:not([hidden])'); i++) await new Promise(r => setTimeout(r, 50)); })()";
    const card = '.main>.card[data-show="f"]';
    return [['eyebrow', ' .cardhead .eyebrow', '.room-card .eyebrow', ['fontSize', 'fontWeight', 'color'], ['height', 'fontSize', 'fontWeight', 'color']],
      ['title', ' .cardhead h2', '.room-card #selected-agent', ['fontSize', 'fontWeight', 'color'], ['fontSize', 'fontWeight', 'color']],
      ['row', ' .row', '.verify-row', ['borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize'], ['height', 'borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize']],
      ['row-title', ' .row .t', '.verify-row .t', ['fontSize', 'fontWeight'], ['fontSize', 'fontWeight']],
      ['row-glyph', ' .row .n', '.verify-row .n', ['borderTopLeftRadius', 'backgroundColor', 'fontSize'], ['width', 'height', 'borderTopLeftRadius', 'backgroundColor', 'fontSize']],
    ].map(([name, mock, app, props, keys]) => ({ name: `verify-card-${name}`, props, keys, pixel: false,
      mock: { url: '?theme=light#6', selector: card + mock }, app: { setup, run, selector: app } }));
  })(),
];
