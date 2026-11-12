// GitHub #432: text and button sizes follow the approved mock element by element. Each entry compares the
// computed font size, weight, colour and (for controls) height of the mock element with its app counterpart.
const T = ['fontSize', 'fontWeight', 'color', 'fontFamily'];
// A control matches the mock when its whole box does: size, padding, line height, layout, border, radius, fill and text.
const B = [...T, 'width', 'height', 'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft', 'lineHeight', 'display', 'columnGap', 'borderTopLeftRadius', 'borderTopWidth', 'borderTopColor', 'backgroundColor'];
const e = (name, keys, mock, app) => ({ name: 'type-' + name, props: keys.filter(k => k !== 'height' && k !== 'width'), keys, pixel: false, mock: { selector: mock }, app: { selector: app } });
module.exports = [
  e('brand', T, '.title-brand strong', '#start-titlebar .title-brand > strong'),
  e('version', T, '.title-brand .ver', '#start-titlebar .app-version'),
  e('title-divider', ['width','height','backgroundColor'], '.title-brand .div', '#start-titlebar .title-divider'),
  { ...e('app-menu-button', B, '#app-menu-button', '#start-app-menu'), pixel: true },
  { ...e('workspace-home', B, '#workspace-home', '#start-workspace-menu'), app: { selector:'#start-workspace-menu', setup:"SIM.reply.workspaces=()=>({name:'Auth Service',current:'/tmp/sim-desktop',registered:[]});" } },
  e('settings-glyph', ['fontSize', 'color'], '#settings-button', '#start-settings-button'),
  e('rail-tab-selected', T, '#rail-agents-tab', '#rail-tab-agents'),
  e('rail-tab-idle', T, '#rail-tasks-tab', '#rail-tab-tasks'),
  e('rail-eyebrow', T, '.railhead .eyebrow', '.startup-team .eyebrow'),
  e('rail-title', T, '.railhead h2', '.startup-team h2'),
  e('import-row-box', ['height', 'columnGap', 'alignItems', 'flexWrap', 'display'], '.goal-import-actions', '#start-mcp-controls'),
  e('import-preview-button', B.filter(k => k !== 'width'), '.goal-import-actions .btn:nth-child(3)', '#start-mcp-preview'),
  e('rail-head-box', ['width', 'height'], '.railhead', '#rail-pane-agents .section-title'),
  e('rail-add-box', B, '.railhead .add', '#start-add'),
  e('rail-sub-box', ['width', 'height'], '.agent-panel .sub', '#rail-pane-agents p.small'),
  // GitHub #869: compare the non-coordinator codex card; the first cards carry different work-state tags.
  e('agent-card-box', B, '.agent:has([data-sessions="codex"])', '.startup-profile[data-agent="codex"]'),
  // Names are operator data: compare the same registered ID to avoid a false narrow-screen wrap.
  { ...e('agent-top-box', ['width', 'height'], '.agent .top', '.startup-profile .profile-top'), mock:{selector:'.agent .top',run:"document.querySelector('.agent .name').textContent='claude';"} },
  { ...e('agent-name-box', ['width', 'height', 'fontSize'], '.agent .name', '.startup-profile .name-row'), mock:{selector:'.agent .name',run:"document.querySelector('.agent .name').textContent='claude';"} },
  e('agent-facts-box', ['width', 'height', 'columnGap', 'rowGap'], '.agent:has([data-sessions="codex"]) .facts:not(.work-facts)', '.startup-profile[data-agent="codex"] .agent-facts:not(.work-facts)'),
  // GitHub #828/#869: the session chip has its own entry (agent-sessions-chip); this compares the models tag.
  e('agent-fact-box', B.filter(k => k !== 'width'), '.agent .fact.models', '.startup-profile .fact.models'),
  e('agent-tools-box', ['width', 'height', 'columnGap'], '.agent .tools', '.startup-profile .profile-tools'),
  e('agent-tool-edit', B, '.agent .tools button', '.startup-profile .profile-tools button'),
  e('agent-tool-remove', B, '.agent .tools .agent-remove', '.startup-profile .remove-agent'),
  e('rail-add', ['fontSize', 'color'], '.railhead .add', '#start-add'),
  e('rail-sub', ['fontSize', 'color'], '.agent-panel .sub', '#rail-pane-agents p.small'),
  e('agent-name', T, '.agent .name', '.startup-profile .name-row strong'),
  e('agent-meta', ['fontSize', 'color'], '.agent .meta', '.startup-profile .agent-meta'),
  e('phase-pill', ['fontSize', 'color'], '.goal-info>.pill', '#start-phase'),
  e('step-number', ['fontSize', 'fontWeight'], '.step-group .tab .num', '.step-group .tab-number'),
  e('step-label', ['fontSize', 'fontWeight'], '.step-group .tab-label', '.step-group button>span:last-child'),
  e('step-card-title', T, '.cardhead h2', '#start-settings-title'),
  e('field-label', ['fontSize', 'fontWeight', 'color'], '.workspace-fields label', '.field-name label'),
  e('workspace-eyebrow', T, '.workspace-config-card .cardhead .eyebrow', '#start-panel-workspace .cardhead .eyebrow'),
  e('workspace-description', [...T,'lineHeight'], '.workspace-config-card .cardhead .sub', '#start-panel-workspace .cardhead .sub'),
  e('workspace-section-title', T, '.workspace-config-section .config-section-head h3', '#start-workspace-section .config-section-head h3'),
  e('workspace-section-note', T, '.workspace-config-section .config-section-head>span', '#start-workspace-section .config-section-head>span'),
  e('goal-section-title', T, '.workspace-goal-section .config-section-head h3', '#start-goal-section .config-section-head h3'),
  e('goal-section-note', T, '.workspace-goal-section .config-section-head p', '#start-goal-section .config-section-head p'),
  e('goal-objective-label', T, '.goal-input-label', '#start-goal-section .goal-input-label'),
  e('workspace-path-note', T, '.workspace-path-note', '#start-workspace-section .workspace-path-note'),
  e('goal-context-title', T, '#workspace-current-goal', '#start-current-goal'),
  e('goal-context-state', T, '#workspace-current-goal-state', '#start-current-goal-state'),
  e('workspace-briefing-button', B, '#team-brief-button', '#brief-team'),

  { name:'workspace-section-box', props:['paddingTop','paddingBottom','borderBottomWidth','borderBottomColor'], keys:['paddingTop','paddingBottom','borderBottomWidth','borderBottomColor'],pixel:false,mock:{selector:'.workspace-config-section'},app:{selector:'#start-workspace-section'} },
  { name:'goal-context-box',props:['paddingTop','paddingRight','paddingBottom','paddingLeft','backgroundColor','borderTopLeftRadius','columnGap'],keys:['paddingTop','paddingRight','paddingBottom','paddingLeft','backgroundColor','borderTopLeftRadius','columnGap'],pixel:false,mock:{selector:'.goal-selection-context'},app:{selector:'#start-goal-section .goal-selection-context'} },

  e('browse', B, '.project-input .btn', '#start-browse'),
  e('goal-list-button', B, '#goal-picker', '#start-goal-list-open'),
  e('new-goal-button', B, '#workspace-new-goal', '#start-new-goal'),
  e('import-button', B, '.goal-import-actions .btn:nth-child(2)', '#start-jira-reference'),
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
        const f = parseFloat(getComputedStyle(e).fontSize), narrowVersion = innerWidth <= 900 && e.matches('.app-version') && f === 9; if (!ok.has(f) && !narrowVersion) bad.push((e.id || e.className) + ':' + f); }
      window.__offScale = bad; return bad.length === 0; })()` })),
  // Hover treatment (preview.css): the same fill / colour / border appears under the pointer.
  ...[['workspace-home', '#workspace-home', '#start-workspace-menu'], ['titlebar-icon', '#settings-button', '#start-settings-button'],
    ['goal-list-button', '#goal-picker', '#start-goal-list-open'], ['agent-remove', '.agent .tools button', '.profile-tools .remove-agent']
  ].map(([name, mock, app]) => ({ name: 'hover-' + name, hover: true, props: ['backgroundColor', 'color', 'borderTopColor'], keys: name === 'agent-remove' ? ['backgroundColor', 'color', 'borderTopColor'] : ['backgroundColor', 'color'], pixel: false,
    mock: { selector: mock, index: name === 'agent-remove' ? 1 : 0 }, app: { selector: app } })),
  // Keyboard focus ring (preview.css): 2px accent outline.
  ...[['titlebar-icon', '#settings-button', '#start-settings-button'], ['rail-tab', '#rail-agents-tab', '#rail-tab-agents']
  ].map(([name, mock, app]) => ({ name: 'focus-' + name, focus: true, props: ['outlineStyle', 'outlineWidth', 'outlineColor', 'outlineOffset'], keys: ['outlineStyle', 'outlineWidth', 'outlineColor', 'outlineOffset'], pixel: false,
    mock: { selector: mock }, app: { selector: app } })),
  // Form controls: text field, choice field (a select in the app) and goal textarea.
  { name: 'type-field-input', props: ['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], keys: [...['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], 'height'], pixel: false, mock: { selector: '.workspace-fields input' }, app: { selector: '#start-name' } },
  { name: 'type-field-choice', props: ['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], keys: [...['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], 'height'], pixel: false, mock: { selector: '.assignment-choice' }, app: { selector: '#start-coordinator + .ac-choice-trigger' } },
  { name: 'type-field-textarea', props: ['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], keys: ['fontSize', 'fontWeight', 'color', 'paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'backgroundColor'], pixel: false, mock: { selector: '#goal-entry' }, app: { selector: '#start-goal' } },
  // Containers: the step card, the status bar and the card's action row.
  { name: 'box-step-card', props: ['borderTopLeftRadius', 'borderBottomLeftRadius', 'borderTopWidth', 'backgroundColor', 'paddingTop', 'paddingLeft', 'boxShadow'], keys: ['borderTopLeftRadius', 'borderBottomLeftRadius', 'borderTopWidth', 'backgroundColor', 'paddingTop', 'paddingLeft', 'boxShadow'], pixel: false, mock: { selector: 'section.card' }, app: { selector: '.step-card' } },
  { name: 'box-statusbar', props: ['backgroundColor', 'borderTopWidth', 'borderTopColor', 'paddingLeft'], keys: ['height', 'backgroundColor', 'borderTopWidth', 'borderTopColor', 'paddingLeft'], pixel: false, mock: { selector: '.status' }, app: { selector: 'footer.statusbar' } },
  { name: 'box-step-tabs', props: ['borderTopLeftRadius', 'backgroundColor'], keys: ['height', 'borderTopLeftRadius', 'backgroundColor'], pixel: false, mock: { selector: '.step-group', index: 0 }, app: { selector: '.step-group', index: 0 } },
  // Vertical rhythm of the Workspace screen: the tab strip is 57px below the title and the card hangs from it (measured on the mock).
  // Position inside the window: offset of each landmark from the title bar's top-left corner, as on the mock.
  ...[['heading', '.main h1', '.startup-heading h1'], ['tabs', '.tabs', '#start-step-tabs'], ['card', '.card', '.step-card'], ['rail', '.rail', '.startup-team']]
    .map(([n, m, a]) => ({ name: 'pos-' + n, props: [], keys: ['dt', 'dl'], pixel: false, mock: { selector: m, rel: '.titlebar' }, app: { selector: a, rel: 'header.titlebar' } })),
  { name: 'box-tabs-below-heading', props: [], keys: ['dy'], pixel: false, mock: { selector: '.tabs', rel: '.main h1' }, app: { selector: '#start-step-tabs', rel: '.startup-heading h1' } },
  { name: 'box-card-hangs-from-tabs', props: [], keys: ['dy'], pixel: false, mock: { selector: '.card', rel: '.tabs' }, app: { selector: '.step-card', rel: '#start-step-tabs' } },
  { name: 'box-titlebar-surface', props: ['backgroundColor', 'borderBottomWidth', 'borderBottomColor', 'borderRightWidth', 'borderRightColor'], keys: ['backgroundColor', 'borderBottomWidth', 'borderBottomColor', 'borderRightWidth', 'borderRightColor'], pixel: false, mock: { selector: '.titlebar' }, app: { selector: 'header.titlebar' } },
  { name: 'box-rail-surface', props: ['backgroundColor', 'borderBottomWidth', 'borderBottomColor', 'borderRightWidth', 'borderRightColor'], keys: ['backgroundColor', 'borderBottomWidth', 'borderBottomColor', 'borderRightWidth', 'borderRightColor'], pixel: false, mock: { selector: '.rail' }, app: { selector: '.startup-team' } },
  { name: 'box-main-surface', props: ['backgroundColor', 'paddingLeft', 'paddingTop'], keys: ['paddingLeft'], pixel: false, mock: { selector: 'main.main' }, app: { selector: '.startup-main' } },
  // Step captions retain full titles when a narrow viewport requires ellipsis.
  { name: 'step-tab-captions', app: {}, assert: `(() => { const tabs = [...document.querySelectorAll('#start-step-tabs [role=tab]')], names = tabs.map(t => t.querySelector('span:last-child').textContent.trim());
    return names.slice(0, 3).join('|') === ['워크스페이스', '에이전트 역할', '계획'].join('|') && tabs.every((t,i) => t.title === names[i] && getComputedStyle(t.querySelector('span:last-child')).textOverflow === 'ellipsis') })()` },
  // "Discard changes" is a step-footer button (mock `.actions .btn.q`) that follows the open step.
  e('discard-button', B, '.actions .btn.q', '#start-discard'),
  { name: 'discard-follows-step', app: { run: "document.getElementById('start-tab-team').click()" }, assert: `(() => { const d = document.getElementById('start-discard'); return !d.closest('header') && d.closest('.step-card') === document.getElementById('start-panel-team') })()` },
  // Rail width by viewport (mock): 250px, 230px below 900px, 190px below 600px.
  ...[[1280, 250], [840, 230], [560, 190]].map(([w, rail]) => ({ name: 'rail-width-at-' + w, viewport: [w, 900], app: {}, assert: `Math.round(document.querySelector('.startup-team').getBoundingClientRect().width) === ${rail}` })),
  // Room (run steps): same elements as the mock's task view, measured in the simulator's running run.
  ...(() => { const room = { setup: "SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.plan.tasks[0].agent='claude';", run: "document.getElementById('start-tab-tasks').click()" };
    const r = (name, keys, mock, app) => ({ name: 'room-' + name, props: keys.filter(k => k !== 'height' && k !== 'width'), keys, pixel: false, mock: { url: '?theme=light#3', selector: mock }, app: { ...room, selector: app } });
    return [r('goal-title', T, '#saved-goal-title', '#goal'), r('task-title', T, '.cardhead h2', '#selected-agent'), r('task-status', ['fontSize', 'fontWeight'], '.cardhead .pill', '#task-status'),
      r('view-tab-selected', ['fontSize', 'fontWeight'], '.ttabs button.on', '.view-tabs .tab.active'), r('view-tab-idle', ['fontSize', 'fontWeight'], '.ttabs span', '.view-tabs .tab:not(.active)'),
      r('composer-label', T, '.dir .eyebrow', '.composer label>span'), r('send-button', B, '.dir .btn.p', '#send-feedback'), r('stop-button', B, '.dir .btn.q', '#stop')]; })(),
  // Quota ring and popup (GitHub #426): icon geometry and pixels, popup geometry and pixels, healthy and error.
  { name: 'quota-ring-healthy', props: ['borderTopLeftRadius', 'color'], keys: ['width', 'height', 'borderTopLeftRadius', 'color'], mock: { selector: '.usage-button', index: 0 }, app: { selector: '.usage-button', index: 0 } },
  { name: 'quota-ring-error', props: ['borderTopLeftRadius', 'color'], keys: ['width', 'height', 'borderTopLeftRadius', 'color'], mock: { selector: '.usage-button.error' }, app: { selector: '.usage-button.error' } },
  // The popup is a fixed overlay that does not depend on the window width; at 840px Chrome rasterises its text half a pixel off for the same boxes, so it is gated at 1280px.
  { name: 'quota-popup-healthy', props: ['borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'backgroundColor'], keys: ['width', 'borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'backgroundColor'],
    mock: { run: "document.querySelector('.agent .name').textContent='claude';document.querySelector('[data-usage=claude]').click();document.head.append(Object.assign(document.createElement('style'), { textContent: 'html,body{background:#fff!important}*{visibility:hidden!important}#usage-popover,#usage-popover *{visibility:visible!important}' }));", selector: '#usage-popover' }, app: { run: "document.querySelectorAll('.usage-button')[0].click();document.head.append(Object.assign(document.createElement('style'), { textContent: 'html,body{background:#fff!important}*{visibility:hidden!important}#usage-popover,#usage-popover *{visibility:visible!important}' }));", selector: '#usage-popover' } },
  // The error popup compares its error box (the mock's buttons render in the UA font, so the footer width differs by design).
  { name: 'quota-popup-error', props: ['borderTopLeftRadius', 'backgroundColor', 'color', 'paddingTop', 'fontSize'], keys: ['width', 'height', 'borderTopLeftRadius', 'backgroundColor', 'color', 'paddingTop', 'fontSize'], pixel: false,
    mock: { run: "document.querySelector('[data-usage=codex]').click()", selector: '.usage-error' }, app: { run: "document.querySelector('.usage-button.error').click()", selector: '.usage-error' } },
  // Quota popup behaviour: opens beside the ring, lists the windows, Escape closes and restores focus; unsupported and error variants.
  { name: 'quota-popup-behaviour', app: { run: "document.querySelectorAll('.usage-button')[0].click()" }, assert: `(() => { const p = document.getElementById('usage-popover'), b = document.querySelectorAll('.usage-button')[0];
    const open = !p.hidden && b.getAttribute('aria-expanded') === 'true' && p.querySelectorAll('.usage-item').length === 2 && p.querySelectorAll('.usage-item.quota-short, .usage-item.quota-long').length === 2 && p.getBoundingClientRect().left > b.getBoundingClientRect().right;
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' })); return open && p.hidden && b.getAttribute('aria-expanded') === 'false' && document.activeElement === b })()` },
  { name: 'quota-popup-unsupported', app: { setup: "SIM.home.quota_indicators.claude = { status: 'unsupported', rings: [], windows: [], budgets: [], error: null, observed_at: 0 };", run: "document.querySelectorAll('.usage-button')[0].click()" },
    assert: "(() => { const p = document.getElementById('usage-popover'); return !!p.querySelector('.usage-unavailable') && !document.querySelector('.usage-button .usage-value') && !p.querySelector('.usage-item') })()" },
  { name: 'quota-popup-error-retry', app: { run: "document.querySelector('.usage-button.error').click()" }, assert: "(() => { const p = document.getElementById('usage-popover'); return !!p.querySelector('.usage-error[role=alert]') && p.querySelector('.usage-refresh').textContent.includes('조회') })()" },
  // Agent card header: ring and handle on the avatar's centre line; the handle is as tall as the ring.
  { name: 'type-grip', props: [], keys: ['width', 'height'], pixel: false, mock: { selector: '.agent .grip' }, app: { selector: '.drag-handle' } },
  { name: 'agent-ring-centre-line', props: [], keys: ['dc'], pixel: false, mock: { selector: '.usage-button', index: 0, rel: '.status-icon' }, app: { selector: '.startup-profile .usage-button', index: 0, rel: '.startup-profile .agent-symbol' } },
  { name: 'agent-grip-centre-line', props: [], keys: ['dc'], pixel: false, mock: { selector: '.agent .grip', rel: '.status-icon' }, app: { selector: '.startup-profile .drag-handle', rel: '.startup-profile .agent-symbol' } },
];
