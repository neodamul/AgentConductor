// GitHub #414: one entry per element of the approved design (docs/design/approved-goal-ui). Each issue that
// builds an element adds its entry; `props` are computed-style properties that must match the mock exactly,
// width/height/mark within `tol` px, and the element's screenshot crop may differ by at most `maxPixelDiff`.
const BOX = ['borderTopWidth', 'borderTopLeftRadius'];
// Include the badge outside the avatar border; wrapping the real node preserves its actual pseudo-element.
const attentionBox = selector => `(() => { const e=document.querySelector(${JSON.stringify(selector)}),box=document.createElement('span');box.id='attention-probe';box.style.cssText='display:inline-flex;flex:none;padding:6px;background:transparent';e.replaceWith(box);box.append(e); })()`;
// The mock has no disabled step; its tabs are all reachable, so the app is compared with a run available (disabled steps dim by design).
const STEP_ONE = "document.querySelector('#start-step-tabs [role=tab]').click();";
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
      selector: '.session-choice .session-meta .app-icon.session-symbol', index: 0, mark: 'content' } },
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
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '.session-choice .session-status' } },
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
  { name: 'about-dialog', props: [], keys: ['width', 'height'],
    mock: { url: '?theme=light&about=update#0', setup: 'referenceDialogResize.disconnect();referenceDialogOpen.disconnect();', selector: '#about-dialog' },
    app: { setup: "SIM.reply.about_status=()=>({version:'0.2.0',show:true,updated:true});", selector: '#about-dialog' } },
  { name: 'step-number-box', props: [], keys: ['width', 'height'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.step-group .tab .num', index: 0 }, app: { selector: '#start-step-tabs .tab-number', index: 0 } },
  { name: 'step-group-icon', props: [], keys: ['width', 'height'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.tabs-group-name svg', index: 0 }, app: { selector: '#start-step-tabs .tabs-group-name svg', index: 0 } },
  { name: 'step-group-label', props: ['fontSize', 'fontWeight', 'color'], keys: ['width', 'height', 'fontSize', 'fontWeight', 'color'],
    mock: { url: '?theme=light#0', selector: '.tabs-group-name .group-label', index: 0 }, app: { selector: '#start-step-tabs .tabs-group-name .group-label', index: 0 } },
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
];
