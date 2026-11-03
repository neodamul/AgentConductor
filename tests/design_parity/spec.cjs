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
];
