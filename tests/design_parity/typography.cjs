// GitHub #432: text and button sizes follow the approved mock element by element. Each entry compares the
// computed font size, weight, colour and (for controls) height of the mock element with its app counterpart.
const T = ['fontSize', 'fontWeight', 'color'], B = [...T, 'height', 'paddingLeft', 'borderTopLeftRadius', 'borderTopWidth', 'borderTopColor', 'backgroundColor'];
const e = (name, keys, mock, app) => ({ name: 'type-' + name, props: keys.filter(k => k !== 'height'), keys, pixel: false, mock: { selector: mock }, app: { selector: app } });
module.exports = [
  e('brand', T, '.title-brand strong', 'header.titlebar > strong'),
  e('workspace-home', ['fontSize', 'height'], '#workspace-home', '#start-workspace-menu'),
  e('settings-glyph', ['fontSize', 'color'], '#settings-button', '#start-settings-button'),
  e('rail-tab-selected', T, '#rail-agents-tab', '#rail-tab-agents'),
  e('rail-tab-idle', T, '#rail-tasks-tab', '#rail-tab-tasks'),
  e('rail-eyebrow', T, '.railhead .eyebrow', '.startup-team .eyebrow'),
  e('rail-title', T, '.railhead h2', '.startup-team h2'),
  e('rail-add', ['fontSize', 'color'], '.railhead .add', '#start-add'),
  e('rail-sub', ['fontSize', 'color'], '.agent-panel .sub', '#rail-pane-agents p.small'),
  e('agent-name', T, '.agent .name', '.startup-profile .name-row strong'),
  e('agent-meta', ['fontSize', 'color'], '.agent .meta', '.startup-profile .agent-meta'),
  e('phase-pill', ['fontSize', 'color'], '.head .pill', '#start-phase'),
  e('step-number', ['fontSize', 'fontWeight'], '.step-group .tab .num', '.step-group .tab-number'),
  e('step-label', ['fontSize', 'fontWeight'], '.step-group .tab-label', '.step-group button>span:last-child'),
  e('step-card-title', T, '.cardhead h2', '#start-settings-title'),
  e('field-label', ['fontSize', 'fontWeight', 'color'], '.workspace-fields label', '.field-name label'),
  e('browse', B, '.project-input .btn', '#start-browse'),
  e('goal-list-button', B, '#workspace-goals-button', '#start-goal-list-open'),
  e('new-goal-button', B, '#workspace-new-goal', '#start-new-goal'),
  e('import-button', B, '.goal-import-actions .btn', '#start-jira-reference'),
  e('save-button', B, '.actions .btn.p', '#start-save'),
  e('statusbar', ['fontSize', 'color'], '.status span', '#start-status'),
  // Every visible text in each screen state uses one size of the mock's scale (no 15/19/20/22/26px).
  ...[['workspace', '', ''], ['room', "SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.plan.tasks[0].agent='claude';", "document.getElementById('start-tab-tasks').click()"],
    ['team', '', "document.getElementById('start-tab-team').click()"], ['review', '', "document.getElementById('start-tab-review').click()"],
    ['settings', '', "document.getElementById('start-settings-button').click()"], ['goal-list', '', "document.getElementById('start-goal-list-open').click()"],
    ['chat', '', "document.getElementById('start-chat-button').click()"]].map(([state, setup, run]) => ({
    name: 'type-scale-' + state, app: { setup, run },
    assert: `(() => { const ok = new Set([10, 10.5, 11, 11.5, 12, 12.5, 13, 14, 16, 17, 18]); const bad = [];
      for (const e of document.querySelectorAll('body *')) { const r = e.getBoundingClientRect(); if (!r.width || !r.height || !(e.matches('input:not([type=checkbox]),textarea,select') || [...e.childNodes].some(n => n.nodeType == 3 && n.textContent.trim()))) continue;
        const f = parseFloat(getComputedStyle(e).fontSize); if (!ok.has(f)) bad.push((e.id || e.className) + ':' + f); }
      window.__offScale = bad; return bad.length === 0; })()` })),
  // Hover treatment (preview.css): the same fill / colour / border appears under the pointer.
  ...[['workspace-home', '#workspace-home', '#start-workspace-menu'], ['titlebar-icon', '#settings-button', '#start-settings-button'],
    ['goal-list-button', '#workspace-goals-button', '#start-goal-list-open'], ['agent-remove', '.agent .tools button', '.profile-tools .remove-agent']
  ].map(([name, mock, app]) => ({ name: 'hover-' + name, hover: true, props: ['backgroundColor', 'color', 'borderTopColor'], keys: name === 'agent-remove' ? ['backgroundColor', 'color', 'borderTopColor'] : ['backgroundColor', 'color'], pixel: false,
    mock: { selector: mock, index: name === 'agent-remove' ? 1 : 0 }, app: { selector: app } })),
  // Keyboard focus ring (preview.css): 2px accent outline.
  ...[['titlebar-icon', '#settings-button', '#start-settings-button'], ['rail-tab', '#rail-agents-tab', '#rail-tab-agents']
  ].map(([name, mock, app]) => ({ name: 'focus-' + name, focus: true, props: ['outlineStyle', 'outlineWidth', 'outlineColor', 'outlineOffset'], keys: ['outlineStyle', 'outlineWidth', 'outlineColor', 'outlineOffset'], pixel: false,
    mock: { selector: mock }, app: { selector: app } })),
  // Form controls: text field, choice field (a select in the app) and goal textarea.
  { name: 'type-field-input', props: ['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], keys: [...['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], 'height'], pixel: false, mock: { selector: '.workspace-fields input' }, app: { selector: '#start-name' } },
  { name: 'type-field-choice', props: ['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], keys: [...['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], 'height'], pixel: false, mock: { selector: '.assignment-choice' }, app: { selector: '#start-coordinator' } },
  { name: 'type-field-textarea', props: ['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], keys: ['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], pixel: false, mock: { selector: '#goal-entry' }, app: { selector: '#start-goal' } },
  // Containers: the step card, the status bar and the card's action row.
  { name: 'box-step-card', props: ['borderTopLeftRadius', 'borderBottomLeftRadius', 'borderTopWidth', 'backgroundColor', 'paddingTop', 'paddingLeft', 'boxShadow'], keys: ['borderTopLeftRadius', 'borderBottomLeftRadius', 'borderTopWidth', 'backgroundColor', 'paddingTop', 'paddingLeft', 'boxShadow'], pixel: false, mock: { selector: 'section.card' }, app: { selector: '.step-card' } },
  { name: 'box-statusbar', props: ['backgroundColor', 'borderTopWidth', 'borderTopColor', 'paddingLeft'], keys: ['height', 'backgroundColor', 'borderTopWidth', 'borderTopColor', 'paddingLeft'], pixel: false, mock: { selector: '.status' }, app: { selector: 'footer.statusbar' } },
  { name: 'box-step-tabs', props: ['borderTopLeftRadius', 'backgroundColor'], keys: ['height', 'borderTopLeftRadius', 'backgroundColor'], pixel: false, tol: 1, mock: { selector: '.step-group', index: 0 }, app: { selector: '.step-group', index: 0 } },
  // Vertical rhythm of the Workspace screen: the tab strip is 57px below the title and the card hangs from it (measured on the mock).
  { name: 'box-heading-tabs-card', app: {}, assert: `(() => { const h = document.querySelector('.startup-heading h1').getBoundingClientRect(), t = document.getElementById('start-step-tabs').getBoundingClientRect(), c = document.querySelector('.step-card').getBoundingClientRect();
    return Math.abs((t.top - h.bottom) - 57) <= 2 && Math.abs(c.top - t.bottom) <= 1 })()` },
  { name: 'box-titlebar-surface', props: ['backgroundColor', 'borderBottomWidth', 'borderBottomColor', 'borderRightWidth', 'borderRightColor'], keys: ['backgroundColor', 'borderBottomWidth', 'borderBottomColor', 'borderRightWidth', 'borderRightColor'], pixel: false, mock: { selector: '.titlebar' }, app: { selector: 'header.titlebar' } },
  { name: 'box-rail-surface', props: ['backgroundColor', 'borderBottomWidth', 'borderBottomColor', 'borderRightWidth', 'borderRightColor'], keys: ['backgroundColor', 'borderBottomWidth', 'borderBottomColor', 'borderRightWidth', 'borderRightColor'], pixel: false, mock: { selector: '.rail' }, app: { selector: '.startup-team' } },
  { name: 'box-main-surface', props: ['backgroundColor', 'paddingLeft', 'paddingTop'], keys: ['paddingLeft'], pixel: false, mock: { selector: 'main.main' }, app: { selector: '.startup-main' } },
];
