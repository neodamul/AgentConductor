// Extracted from spec.cjs to keep each comparison file under 300 lines.
const {STEPS_ENABLED}=require('./spec_context.cjs');
module.exports=[
  // #598: the Room's review card (mock view #7 = "c"): coordinator view once the run waits for approval.
  ...(() => {
    const setup = STEPS_ENABLED + "SIM.room.report.state='waiting_approval';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'abcdef1234567890'};SIM.room.report.tests=[{task:'task-1',exit_code:0,timed_out:false,output:''}];SIM.reply.evidence=()=>({revision:'x1',report:SIM.room.report,truncated:false,test_output:[{task:'task-1',exit_code:0,timed_out:false,stdout:'',stderr:''}],diff:'diff --git a/src/auth/refresh.rs b/src/auth/refresh.rs\\n--- a/src/auth/refresh.rs\\n+++ b/src/auth/refresh.rs\\n@@ -1 +1 @@\\n-a\\n+b\\n+c\\ndiff --git a/docs/security.md b/docs/security.md\\n+++ b/docs/security.md\\n+x\\n'});";
    const run = "document.querySelectorAll('#start-step-tabs [role=tab]')[5].click(); (async () => { for (let i = 0; i < 200 && !document.querySelector('.final-step .session-choice'); i++) await new Promise(r => setTimeout(r, 50)); document.querySelector('.final-step .session-choice').click(); for (let i = 0; i < 200 && !document.querySelector('.verify-row, .review-summary:not([hidden])'); i++) await new Promise(r => setTimeout(r, 50)); })()";
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
  // GitHub #706: the verification card's footer and heading line (mock step 5 = "f").
  ...(() => {
    const setup = STEPS_ENABLED + "SIM.room.report.state='running';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.tests=[{task:'task-1',exit_code:0,timed_out:false,output:''}];";
    const run = "document.querySelectorAll('#start-step-tabs [role=tab]')[4].click(); (async () => { for (let i = 0; i < 200 && !document.querySelector('.final-step .session-choice'); i++) await new Promise(r => setTimeout(r, 50)); document.querySelector('.final-step .session-choice').click(); for (let i = 0; i < 200 && !document.querySelector('.verify-row'); i++) await new Promise(r => setTimeout(r, 50)); })()";
    const card = '.main>.card[data-show="f"]';
    return [['note', ' .actions .note', '#verify-footer .note', ['fontSize', 'color'], ['fontSize', 'color']],
      ['stop', ' .actions .btn.q', '#verify-stop', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius']],
      ['wait', ' .actions .btn.p', '#verify-wait', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius']],
    ].map(([name, mock, app, props, keys]) => ({ name: `verify-footer-${name}`, props, keys, pixel: false,
      mock: { url: '?theme=light#6', selector: card + mock }, app: { setup, run, selector: app } }))
      .concat([{ name: 'verify-no-tabs', assert: "(() => { const c = document.querySelector('#run-screen .room-card'); return c.classList.contains('verifying') && getComputedStyle(c.querySelector('.view-tabs')).display === 'none' && getComputedStyle(c.querySelector('.composer')).display === 'none' && !document.getElementById('verify-footer').hidden && document.getElementById('run-pill').textContent.length > 0; })()", app: { setup, run } }]);
  })(),
  // GitHub #689: the approval footer and the heading line under the title (mock view #7 = "c").
  ...(() => {
    const setup = STEPS_ENABLED + "SIM.room.report.state='waiting_approval';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'abcdef1234567890'};SIM.room.report.tests=[{task:'task-1',exit_code:0,timed_out:false,output:''}];SIM.reply.evidence=()=>({revision:'x1',report:SIM.room.report,truncated:false,test_output:[],diff:''});";
    const run = "document.querySelectorAll('#start-step-tabs [role=tab]')[5].click(); (async () => { for (let i = 0; i < 200 && !document.querySelector('.final-step .session-choice'); i++) await new Promise(r => setTimeout(r, 50)); document.querySelector('.final-step .session-choice').click(); for (let i = 0; i < 200 && !document.querySelector('.verify-row, .review-summary:not([hidden])'); i++) await new Promise(r => setTimeout(r, 50)); })()";
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
  // GitHub #751: an open group shows a submenu popup with the same box and rows as the mock.
  ...[
    { name: 'app-submenu-box', props: ['borderTopLeftRadius', 'paddingTop', 'borderTopWidth', 'backgroundColor'], keys: ['borderTopLeftRadius', 'paddingTop', 'borderTopWidth', 'backgroundColor'], mock: '#app-menu .submenu:not([hidden])', app: '#start-workspace-list .submenu:not([hidden])' },
    { name: 'app-submenu-item', props: ['borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize', 'color'], keys: ['height', 'borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize', 'color'], mock: '#app-menu .submenu:not([hidden]) button', app: '#start-workspace-list .submenu:not([hidden]) button' },
  ].map(({ mock, app, ...rest }) => ({ ...rest, pixel: false,
    mock: { url: '?theme=light#0', run: "document.getElementById('app-menu-button').click(); document.querySelector('#app-menu .menu-parent').click()", selector: mock },
    app: { run: "(async () => { document.getElementById('start-app-menu').click(); await new Promise(r => setTimeout(r, 900)); document.querySelector('#start-workspace-list .menu-parent').click(); })()", selector: app } })),
  // #921: Help lists Release notes second, right after Check for updates, with the same row style.
  { name: 'app-help-release-notes', props: ['borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize', 'color'], keys: ['height', 'borderTopLeftRadius', 'paddingTop', 'paddingLeft', 'fontSize', 'color'], pixel: false,
    mock: { url: '?theme=light#0', run: "document.getElementById('app-menu-button').click(); document.querySelector('[aria-controls=submenu-help]').click()", selector: '#submenu-help > button', index: 1 },
    app: { run: "(async () => { document.getElementById('start-app-menu').click(); await new Promise(r => setTimeout(r, 900)); [...document.querySelectorAll('#start-workspace-list .menu-parent')].at(-1).click(); })()", selector: '#start-workspace-list .submenu:not([hidden]) > button', index: 1 } },
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
  { name: 'view-tab', props: ['borderTopLeftRadius', 'borderBottomLeftRadius', 'fontSize', 'fontWeight', 'lineHeight', 'paddingLeft', 'marginBottom', 'borderTopWidth', 'borderBottomWidth', 'backgroundColor', 'color'], keys: ['height', 'borderTopLeftRadius', 'borderBottomLeftRadius', 'fontSize', 'fontWeight', 'lineHeight', 'paddingLeft', 'marginBottom', 'borderTopWidth', 'borderBottomWidth', 'backgroundColor', 'color'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.ttabs span', index: 1 },
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '.view-tabs .tab', index: 1 } },
  // GitHub #815: the selected tab and the row's baseline match the mock's folder tabs.
  { name: 'view-tab-selected', props: ['borderTopLeftRadius', 'borderBottomLeftRadius', 'fontSize', 'fontWeight', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderBottomColor', 'backgroundColor', 'color'], keys: ['height', 'borderTopLeftRadius', 'borderBottomLeftRadius', 'fontSize', 'fontWeight', 'paddingLeft', 'borderTopWidth', 'borderTopColor', 'borderBottomColor', 'backgroundColor', 'color'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.ttabs .on' }, app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '.view-tabs .tab.active' } },
  { name: 'view-tabs-baseline', props: ['borderBottomWidth', 'borderBottomColor', 'columnGap'], keys: ['borderBottomWidth', 'borderBottomColor', 'columnGap'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.ttabs' }, app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '.view-tabs' } },
  // GitHub #817: in the Room the left panel and its selected tab share the mock's panel color.
  { name: 'room-rail-background', props: ['backgroundColor'], keys: ['backgroundColor'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.rail' }, app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '#run-screen .rail' } },
  { name: 'room-rail-tab-selected', props: ['backgroundColor', 'borderTopColor', 'color'], keys: ['backgroundColor', 'borderTopColor', 'color'], pixel: false,
    mock: { url: '?theme=light#3', selector: '.rail-tabs button[aria-selected="true"]' }, app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '#rail-tabs [role=tab][aria-selected=true]' } },
  { name: 'room-rail-tab-matches-panel', assert: "(() => { const t = document.querySelector('#rail-tabs [role=tab][aria-selected=true]'), r = t.getBoundingClientRect(), behind = document.elementsFromPoint(r.left + r.width / 2, r.bottom + 30).find(e => getComputedStyle(e).backgroundColor !== 'rgba(0, 0, 0, 0)'); return document.body.classList.contains('on-run') && getComputedStyle(t).backgroundColor === getComputedStyle(behind).backgroundColor; })()",
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()" } },
  // GitHub #824: on the Workspace step the Goal list and team briefing sit in the page heading like the other steps; New Goal stays in the Goal section.
  { name: 'workspace-goal-controls-in-heading', assert: "(() => { const h = document.querySelector('.startup-heading'), g = document.getElementById('start-goal-list-open'), b = document.getElementById('brief-team'), n = document.getElementById('start-new-goal'); return h.contains(g) && h.contains(b) && g.nextElementSibling === b && !h.contains(n) && g.getBoundingClientRect().width > 0; })()",
    app: { run: "document.getElementById('start-tab-workspace')?.click()" } },
  // GitHub #827: the folded Goal heading's Expand toggle matches the mock.
  { name: 'goal-title-toggle', props: ['fontSize', 'color', 'paddingLeft', 'backgroundColor', 'borderTopWidth'], keys: ['fontSize', 'color', 'paddingLeft', 'backgroundColor', 'borderTopWidth'], pixel: false,
    mock: { url: '?theme=light#3', selector: '#goal-title-toggle' }, app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()", selector: '#goal-toggle' } },
  { name: 'goal-expanded-text', props: ['whiteSpace','maxHeight','overflowY','fontSize','lineHeight'], keys: ['whiteSpace','maxHeight','overflowY','fontSize','lineHeight'], pixel: false,
    mock: { url: '?theme=light#3', run: "document.getElementById('goal-title-toggle').click()", selector: '.goal-title-text' },
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click();document.getElementById('goal-toggle').click()", selector: '#goal-full' } },
];
