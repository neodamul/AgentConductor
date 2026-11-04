// GitHub #414: one entry per element of the approved design (docs/design/approved-goal-ui). Each issue that
// builds an element adds its entry; `props` are computed-style properties that must match the mock exactly,
// width/height/mark within `tol` px, and the element's screenshot crop may differ by at most `maxPixelDiff`.
const BOX = ['borderTopWidth', 'borderTopLeftRadius'];
module.exports = [
  { name: 'agent-avatar', props: BOX, maxPixelDiff: 0.25, // the mock's border carries the agent status colour; the app's arrives with #410 (then 0.02)
   
    mock: { url: '?theme=light#0', selector: '.status-icon', index: 0, mark: '.agent-brand-logo' },
    app: { selector: '.agent-symbol.app-icon', index: 0, mark: 'content' } },
  // The task rail of the run steps: the Room opens through the simulator's run and the Tasks tab.
  { name: 'task-avatar', props: BOX, maxPixelDiff: 0.005,
    mock: { url: '?theme=light#3', selector: '.outline-task .outline-agent', index: 0, mark: '.agent-brand-logo' },
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.plan.tasks[0].agent='claude';", run: "document.getElementById('start-tab-tasks').click()",
      selector: '.session-choice .session-meta .app-icon.session-symbol', index: 0, mark: 'content' } },
  // Shell geometry (GitHub #408). Positions differ because the mock sits in a framed window, so x/y are not compared.
  { name: 'rail', props: [], keys: ['width'], pixel: false,
    mock: { selector: '.rail' }, app: { selector: '.startup-team' } },
  { name: 'rail-tab', props: ['borderTopLeftRadius', 'fontSize'], keys: ['width', 'height', 'borderTopLeftRadius', 'fontSize'], pixel: false,
    mock: { selector: '.rail-tabs button', index: 1 }, app: { selector: '#rail-tabs [role=tab]', index: 1 } },
  { name: 'step-tab', props: ['borderTopLeftRadius'], keys: ['height', 'borderTopLeftRadius'], pixel: false, tol: 1,
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
  { name: 'task-row', props: ['borderTopLeftRadius'], keys: ['width', 'borderTopLeftRadius'], pixel: false, tol: 1.5,
    mock: { url: '?theme=light#3', selector: '.outline-task' },
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '.session' } },
  { name: 'task-status-tag', props: ['borderTopLeftRadius'], keys: ['height', 'borderTopLeftRadius'], pixel: false, tol: 1,
    mock: { url: '?theme=light#3', selector: '.outline-task .outline-state-badge' },
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '.session-choice .session-status' } },
  // Step groups (GitHub #408): a subdued background per Prepare / Run / Finish set with a 14px group icon.
  ...['prepare', 'run', 'finish'].map((group, index) => ({ name: `step-group-${group}`, props: ['borderTopLeftRadius', 'backgroundColor'], keys: ['height', 'borderTopLeftRadius', 'backgroundColor'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.tabs .step-group', index }, app: { selector: `#start-step-tabs .step-group[data-group=${group}]` } })),
  { name: 'step-group-icon', props: [], keys: ['width', 'height'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.tabs-group-name svg', index: 0 }, app: { selector: '#start-step-tabs .tabs-group-name svg', index: 0 } },
  // Step card (GitHub #408): the header and the action footer stay visible while only the body scrolls.
  { name: 'step-card-fixed-header-footer', viewport: [1280, 520],
    app: { run: "const m = document.querySelector('.startup-main'); m.scrollTop = Math.floor(m.scrollHeight / 2)" },
    assert: "(() => { const c = document.querySelector('#start-panel-workspace'), h = c.querySelector('.section-title').getBoundingClientRect(), f = c.querySelector('.startup-actions').getBoundingClientRect(); return h.top >= 0 && h.bottom <= innerHeight && f.top >= 0 && f.bottom <= innerHeight + 1; })()" },
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
    assert: "(() => { const hidden = document.querySelector('.startup-team').getBoundingClientRect().width === 0 && document.querySelector('.startup-main').getBoundingClientRect().width > window.__w + 200; document.getElementById('start-rail-toggle').click(); return hidden && document.querySelector('.startup-team').getBoundingClientRect().width === 250; })()" },
  // Rail width follows the design's breakpoints: 250px, 230px below 900px (GitHub #408).
  { name: 'rail-width-840', viewport: [840, 800], app: {}, assert: "document.querySelector('.startup-team').getBoundingClientRect().width === 230" },
  { name: 'rail-width-1280', app: {}, assert: "document.querySelector('.startup-team').getBoundingClientRect().width === 250" },
  // Result summary (GitHub #408): compact metadata on one ruled line - 16px/650 values, 14px/500 labels.
  { name: 'result-metric-value', props: ['fontSize', 'fontWeight'], keys: ['fontSize', 'fontWeight'], pixel: false,
    mock: { url: '?theme=light#8', selector: '.result-stats strong' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n+1\\n',tests:[{task:'t',exit_code:0}],details:[]})", selector: '.result-metric strong' } },
  { name: 'result-metric-label', props: ['fontSize', 'fontWeight'], keys: ['fontSize', 'fontWeight'], pixel: false,
    mock: { url: '?theme=light#8', selector: '.result-stats span' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n+1\\n',tests:[{task:'t',exit_code:0}],details:[]})", selector: '.result-metric span' } },
  // Full-window behaviour (GitHub #408): a big window reflows the centre; rail, icons, tabs and borders keep their size.
  { name: 'fullscreen-reflow', viewport: [1920, 1080], app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()" },
    assert: "(() => { const r = n => document.querySelector(n).getBoundingClientRect(); const q = n => getComputedStyle(document.querySelector(n)); return r('.rail').width === 250 && r('.session-symbol').width === 28 && r('.view-tabs .tab').height === 29 && q('.session-symbol').borderTopWidth === '1.5px' && r('.center').width > 1200; })()" },
];
