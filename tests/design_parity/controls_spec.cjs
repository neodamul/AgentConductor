// Extracted from spec.cjs to keep each comparison file under 300 lines.
const {STEPS_ENABLED,STEP_REFERENCE_WIDTH}=require('./spec_context.cjs');
module.exports=[
  {name:'sessions-danger-confirm',props:['fontSize','fontWeight','backgroundColor','color','borderTopColor','paddingTop','paddingLeft'],keys:['height','fontSize','fontWeight','backgroundColor','color','borderTopColor','paddingTop','paddingLeft'],pixel:false,
   mock:{url:'?theme=light#1',run:"openSessions('claude');document.getElementById('sessions-clean-finished').click()",selector:'#sessions-confirm-ok'},
   app:{run:"(async()=>{SIM.reply.agent_sessions=()=>({claude:[{run:'r',task:'t',label:'T1 done',goal:'g',state:'done',started_ms:1,last_ms:1,cleanable:true,stoppable:false}]});await acSessions.refresh();acSessions.open('claude');await acSessions.refresh();document.getElementById('start-agent-sessions-clean-finished').click();})()",selector:'#start-agent-sessions-confirm-ok'}},

  // GitHub #826: a changed file row and its unfolded diff match the mock.
  { name: 'result-file-diff-toggle', props: ['fontSize', 'paddingLeft', 'backgroundColor', 'borderTopWidth'], keys: ['fontSize', 'paddingLeft', 'backgroundColor', 'borderTopWidth'], pixel: false,
    mock: { url: '?theme=light#8', run: "document.querySelector('#result-detail details.result-section').open=true;document.querySelector('#result-detail .file-diff-toggle').click()", selector: '#result-detail .file-diff-toggle' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n@@ -1 +1 @@\\n-a\\n+b\\n',tests:[{task:'t',exit_code:0}],details:[]});document.querySelector('#result-dialog details.result-section').open=true;document.querySelector('#result-dialog .file-diff-toggle').click()", selector: '#result-dialog .file-diff-toggle' } },
  { name: 'result-file-diff', props: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'whiteSpace'], keys: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'whiteSpace'], pixel: false,
    mock: { url: '?theme=light#8', run: "document.querySelector('#result-detail details.result-section').open=true;document.querySelector('#result-detail .file-diff-toggle').click()", selector: '#result-detail .file-diff' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n@@ -1 +1 @@\\n-a\\n+b\\n',tests:[{task:'t',exit_code:0}],details:[]});document.querySelector('#result-dialog details.result-section').open=true;document.querySelector('#result-dialog .file-diff-toggle').click()", selector: '#result-dialog .file-diff' } },
  // GitHub #828: the agent card's session chip and a session row match the mock.
  { name: 'agent-sessions-chip', props: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color'], keys: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color'], pixel: false,
    mock: { url: '?theme=light#1', run: "agentSessions.claude=agentSessions.claude.filter(s=>s.state==='done');renderSessionButtons()", selector: '[data-sessions="claude"]' }, app: { run: "(async()=>{SIM.reply.agent_sessions=()=>({claude:[]});await acSessions.refresh();})()", selector: '[data-sessions="claude"]' } },
  // GitHub #887: the API token hint in the MCP server form matches the mock's secondary note.
  { name: 'mcp-secret-hint', props: ['fontSize', 'lineHeight', 'color', 'marginTop', 'marginBottom', 'whiteSpace'], keys: ['fontSize', 'lineHeight', 'color', 'marginTop', 'marginBottom', 'whiteSpace'], pixel: false,
    mock: { url: '?theme=light#1', run: "openMcp();openMcpEditor(null)", selector: '#mcp-secret-hint' },
    app: { run: "(async()=>{document.getElementById('start-settings-button').click();document.getElementById('start-mcp-settings').click();await new Promise(r=>setTimeout(r,300));document.getElementById('start-mcp-add').click();await new Promise(r=>setTimeout(r,100));})()", selector: '#start-mcp-secret-hint' } },
  // GitHub #886: the OpenCode format hint beside the MCP JSON import matches the mock's secondary note.
  { name: 'mcp-import-format-hint', props: ['fontSize', 'lineHeight', 'color', 'marginTop', 'whiteSpace'], keys: ['fontSize', 'lineHeight', 'color', 'marginTop', 'whiteSpace'], pixel: false,
    mock: { url: '?theme=light#1', run: "openMcp();document.getElementById('mcp-advanced').open=true", selector: '#mcp-import-format' },
    app: { run: "(async()=>{document.getElementById('start-settings-button').click();document.getElementById('start-mcp-settings').click();await new Promise(r=>setTimeout(r,300));document.getElementById('start-mcp-advanced').open=true;})()", selector: '#start-mcp-import-format' } },
  // GitHub #869/#870: the coordinator role tag and band, a work-state tag, an info tag and the model list match the mock.
  { name: 'agent-role-tag', props: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color', 'borderTopColor'], keys: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color', 'borderTopColor'], pixel: false,
    mock: { url: '?theme=light#1', selector: '.agent .fact.role' }, app: { selector: '.startup-profile .fact.role' } },
  { name: 'agent-coordinator-band', props: ['boxShadow', 'borderTopColor'], keys: ['boxShadow', 'borderTopColor'], pixel: false,
    mock: { url: '?theme=light#1', selector: '.agent.coordinator' }, app: { selector: '.startup-profile.coordinator' } },
  { name: 'agent-state-tag', props: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color', 'borderTopColor'], keys: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color', 'borderTopColor'], pixel: false,
    mock: { url: '?theme=light#1', selector: '[data-state-tag="running"]' }, app: { run: "(async()=>{SIM.reply.agent_sessions=()=>({claude:[{run:'run-a',task:'t',label:'T1 t',goal:'g',state:'running',started_ms:1,last_ms:1,note:null,cleanable:false,stoppable:true}]});await acSessions.refresh();})()", selector: '[data-state-tag="running"]' } },
  { name: 'agent-info-tag', props: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color', 'borderTopColor'], keys: ['fontSize', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color', 'borderTopColor'], pixel: false,
    mock: { url: '?theme=light#1', selector: '.agent .fact.models' }, app: { selector: '.startup-profile .fact.models' } },
  { name: 'agent-model-list', props: ['paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'borderTopColor', 'fontSize'], keys: ['paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'borderTopColor', 'fontSize'], pixel: false,
    mock: { url: '?theme=light#1', run: "const t=document.querySelector('.agent .fact.models');t.focus();t.dispatchEvent(new FocusEvent('focusin',{bubbles:true}))", selector: '.agent .fact.models .model-list' },
    app: { run: "const t=document.querySelector('.startup-profile .fact.models');t.focus();t.dispatchEvent(new FocusEvent('focusin',{bubbles:true}))", selector: '.startup-profile .fact.models .model-list' } },
  { name: 'agent-session-row', props: ['paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'columnGap'], keys: ['paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'columnGap'], pixel: false,
    mock: { url: '?theme=light#1', run: "openSessions('claude')", selector: '#sessions-list .session-row' }, app: { run: "(async()=>{SIM.reply.agent_sessions=()=>({claude:[{run:'run-a',task:'t1',label:'T1 Rotate tokens',goal:'Rotate tokens',state:'running',started_ms:Date.now(),last_ms:Date.now(),note:null,cleanable:false,stoppable:true}]});await acSessions.refresh();acSessions.open('claude');await acSessions.refresh();})()", selector: '#start-agent-sessions-list .session-row' } },
  { name: 'agent-session-state', props: ['fontSize', 'fontWeight', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color'], keys: ['fontSize', 'fontWeight', 'paddingLeft', 'paddingTop', 'borderTopWidth', 'borderTopLeftRadius', 'backgroundColor', 'color'], pixel: false,
    mock: { url: '?theme=light#1', run: "openSessions('claude')", selector: '#sessions-list .session-state.running' }, app: { run: "(async()=>{SIM.reply.agent_sessions=()=>({claude:[{run:'run-a',task:'t1',label:'T1 Rotate tokens',goal:'Rotate tokens',state:'running',started_ms:Date.now(),last_ms:Date.now(),note:null,cleanable:false,stoppable:true}]});await acSessions.refresh();acSessions.open('claude');await acSessions.refresh();})()", selector: '#start-agent-sessions-list .session-state.running' } },
  { name: 'view-tabs-not-stretched', assert: "(() => { const t = [...document.querySelectorAll('.view-tabs .tab')]; const w = document.querySelector('.view-tabs').getBoundingClientRect().width; return t.length >= 4 && t.every(x => x.getBoundingClientRect().width < w / 3); })()",
    app: { setup: "SIM.home.run_available=true;SIM.home.run_state='running';", run: "document.getElementById('start-tab-tasks').click()" } },
  // Compact Goal header (GitHub #408): 17px semibold single line.
  { name: 'goal-title', props: ['fontSize', 'fontWeight'], keys: ['height', 'fontSize', 'fontWeight'], pixel: false,
    mock: { url: '?theme=light#0', selector: '.head h1' }, app: { selector: '.startup-main .startup-heading h1' } },
  // Left rail toggle (GitHub #408): a 32px titlebar button hides and restores the whole rail; the centre takes the space.
  { name: 'rail-toggle-button', props: ['borderTopLeftRadius'], keys: ['width', 'height', 'borderTopLeftRadius'], pixel: false,
    mock: { url: '?theme=light#0', selector: '#sidebar-toggle' }, app: { selector: '#start-rail-toggle' } },
  // #868: both panel glyphs follow the same 18px approved titlebar size.
  ...[['left', '#sidebar-toggle', '#start-rail-toggle'], ['right', '#right-panel-toggle', '#start-inspector-toggle']].map(([side, mock, app]) => ({
    name: 'panel-toggle-icon-' + side, keys: ['width', 'height'], pixel: false,
    mock: { selector: mock + ' svg' }, app: { selector: app + ' svg' } })),
  { name: 'rail-toggle-behaviour',
    app: { run: "window.__w = document.querySelector('.startup-main').getBoundingClientRect().width; document.getElementById('start-rail-toggle').click()" },
    assert: "(() => { const hidden = document.querySelector('.startup-team').getBoundingClientRect().width === 0 && document.querySelector('.startup-main').getBoundingClientRect().width > window.__w + 200; document.getElementById('start-rail-toggle').click(); return hidden && document.querySelector('.startup-team').getBoundingClientRect().width === (innerWidth < 900 ? 230 : 250); })()" },
  // Rail width follows the design's breakpoints: 250px, 230px below 900px (GitHub #408).
  { name: 'rail-width-840', viewport: [840, 800], app: {}, assert: "document.querySelector('.startup-team').getBoundingClientRect().width === 230" },
  { name: 'rail-width-1280', viewport: [1280, 800], app: {}, assert: "document.querySelector('.startup-team').getBoundingClientRect().width === 250" },
  // Result summary (GitHub #408): compact metadata on one ruled line - 16px/650 values, 14px/500 labels.
  // GitHub #799: the result sections expand and collapse like the mock's.
  { name: 'result-section-summary', props: ['marginTop', 'marginBottom', 'display', 'columnGap', 'cursor'], keys: ['marginTop', 'marginBottom', 'display', 'columnGap', 'cursor'], pixel: false,
    mock: { url: '?theme=light#8', selector: '#result-detail .result-section summary' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n+1\\n',tests:[{task:'t',exit_code:0}],details:[]})", selector: '#result-dialog .result-section summary' } },
  { name: 'result-section-title', props: ['fontSize', 'fontWeight'], keys: ['fontSize', 'fontWeight'], pixel: false,
    mock: { url: '?theme=light#8', selector: '#result-detail .result-section summary h3' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n+1\\n',tests:[{task:'t',exit_code:0}],details:[]})", selector: '#result-dialog .result-section summary h3' } },
  { name: 'result-section-count', props: ['fontSize', 'fontWeight', 'color'], keys: ['fontSize', 'fontWeight', 'color'], pixel: false,
    mock: { url: '?theme=light#8', selector: '#result-detail .result-section-count' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n+1\\n',tests:[{task:'t',exit_code:0}],details:[]})", selector: '#result-dialog .result-section-count' } },
  { name: 'result-section-toggle', props: ['fontSize', 'fontWeight', 'color'], keys: ['fontSize', 'fontWeight', 'color'], pixel: false,
    mock: { url: '?theme=light#8', selector: '#result-detail .result-section-toggle' }, app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n+1\\n',tests:[{task:'t',exit_code:0}],details:[]})", selector: '#result-dialog .result-section-toggle' } },
  { name: 'result-section-toggle-runtime', app: { run: "acResultView.show({goal:'g',branch:'b',repo:'r',commit:'abcdef123456',diff:'diff --git a/x b/x\\n+1\\n',tests:[{task:'t',exit_code:0}],details:[]})" },
    assert: "(async()=>{const sections=[...document.querySelectorAll('#result-dialog .result-section')],wait=()=>new Promise(r=>setTimeout(r,0));if(sections.length!==3||!sections.every((s,i)=>{const h=s.querySelector('summary');return !s.open&&h.getAttribute('aria-expanded')==='false'&&h.querySelector('.result-section-count')?.textContent.trim()===`· ${[1,1,0][i]}`&&h.querySelector('.result-section-toggle')?.textContent===acMessage('room.result_expand');}))return false;const s=sections[0],h=s.querySelector('summary');h.click();await wait();const opened=s.open&&h.getAttribute('aria-expanded')==='true'&&h.querySelector('.result-section-toggle').textContent===acMessage('room.result_collapse');h.click();await wait();return opened&&!s.open&&h.getAttribute('aria-expanded')==='false'&&h.querySelector('.result-section-toggle').textContent===acMessage('room.result_expand');})()" },
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
  // #857: compare the update icon only when a newer release is known; the inspector toggle is always present.
  { name: 'titlebar-update', props: ['borderTopLeftRadius', 'borderTopColor'], keys: ['width', 'height', 'borderTopLeftRadius', 'borderTopColor'], pixel: false,
    mock: { selector: '#update-button' }, app: { setup: "SIM.reply.check_update=SIM.reply.update_status=()=>({phase:'idle',current:'1.0.0',latest:'1.1.0',newer:true});", selector: '#start-update-button' } },
  { name: 'titlebar-inspector-toggle', props: ['borderTopLeftRadius', 'borderTopColor'], keys: ['width', 'height', 'borderTopLeftRadius', 'borderTopColor'], pixel: false,
    mock: { selector: '#right-panel-toggle' }, app: { selector: '#start-inspector-toggle' } },
  // The docked task inspector is available before any run exists and starts closed, like the mock's.
  { name: 'startup-inspector-open', props: ['backgroundColor', 'paddingTop', 'paddingLeft', 'borderLeftWidth'], keys: ['width', 'backgroundColor', 'paddingTop', 'paddingLeft', 'borderLeftWidth'], pixel: false,
    mock: { url: '?theme=light#0', run: STEP_REFERENCE_WIDTH + ";document.getElementById('right-panel-toggle').click()", selector: '#task-inspector-panel' },
    app: { run: "document.getElementById('start-inspector-toggle').click()", selector: '#start-inspector' } },
  // Closed by default, and docked at the right edge of the window once opened (not in the rail or below the card).
  // GitHub #677: the titlebar holds only the mock's controls, on the setup steps and while a run is open.
  { name: 'titlebar-controls', assert: "(() => [...document.querySelectorAll('header.titlebar button')].filter(b => b.getClientRects().length).map(b => b.id).join() === 'start-app-menu,start-rail-toggle,start-workspace-menu,start-chat-button,start-settings-button,start-inspector-toggle')()",
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
  // GitHub #685: the roles step footer (mock step 2 = "i").
  ...[
    ['roles-footer-note', ['fontSize', 'color'], ['fontSize', 'color'], '.card[data-show="i"] .actions .note', '#start-panel-team .startup-actions .note'],
    ['roles-footer-add', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius'], '.card[data-show="i"] .actions .btn.q', '#start-roles-suggest'],
    ['roles-footer-next', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color'], '.card[data-show="i"] .actions .btn.p', '#start-roles-next'],
  ].map(([name, props, keys, mock, app]) => ({ name, props, keys, pixel: false,
    mock: { url: '?theme=light#1', selector: mock },
    app: { setup: STEPS_ENABLED, run: "document.getElementById('start-tab-team').click()", selector: app } })),
  // GitHub #708: the line under the goal title on the plan step (mock step 3).
  { name: 'plan-heading-info', props: ['fontSize', 'color'], keys: ['height', 'fontSize', 'color'], pixel: false,
    mock: { url: '?theme=light#2', selector: '.runmeta[data-show="a"] span' },
    app: { setup: "SIM.home.proposal=SIM.proposal('ready');", run: "document.getElementById('start-tab-review').click()", selector: '#start-goal-info .goal-info-content' } },
  // GitHub #686: the plan card's position line and footer (mock step 3 = "a").
  ...[
    ['plan-position', ['fontSize', 'color'], ['fontSize', 'color'], '.card[data-show="a"] .phase span:first-child', '#start-plan-summary .plan-position'],
    ['plan-footer-note', ['fontSize', 'color'], ['fontSize', 'color'], '.card[data-show="a"] .actions .note', '#start-panel-review .startup-review-footer .note'],
    ['plan-regenerate', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius'], '.card[data-show="a"] .actions .btn.q', '#start-regenerate'],
    ['plan-approve', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color'], '.card[data-show="a"] .actions .btn.p', '#start-run'],
  ].map(([name, props, keys, mock, app]) => ({ name, props, keys, pixel: false,
    mock: { url: '?theme=light#2', selector: mock },
    app: { setup: "SIM.home.proposal=SIM.proposal('ready');", run: "document.getElementById('start-tab-review').click()", selector: app } })),
  // GitHub #724–#726: one-line plan rows, a step card that reaches the status bar and step footers without a divider.
  ...[
    ['plan-row', ['paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopLeftRadius', 'marginTop'], ['paddingTop', 'paddingLeft', 'borderTopWidth', 'borderTopLeftRadius', 'marginTop'], '.card[data-show="a"] .row', '#start-tasks .startup-task'],
    ['plan-row-number', ['fontSize', 'fontWeight', 'borderTopLeftRadius'], ['width', 'height', 'fontSize', 'fontWeight', 'borderTopLeftRadius'], '.card[data-show="a"] .row .n', '#start-tasks .task-num'],
    ['plan-row-title', ['fontSize', 'fontWeight'], ['fontSize', 'fontWeight'], '.card[data-show="a"] .row .t', '#start-tasks .task-oneline'],
    ['plan-row-detail', ['fontSize', 'color'], ['fontSize', 'color'], '.card[data-show="a"] .row .s', '#start-tasks .task-owner'],
    ['plan-footer-box', ['paddingTop', 'borderTopWidth'], ['paddingTop', 'borderTopWidth'], '.card[data-show="a"] > .dir', '#start-panel-review .startup-review-footer'],
    ['plan-regenerate-fill', ['backgroundColor', 'borderTopWidth'], ['backgroundColor', 'borderTopWidth'], '.card[data-show="a"] .actions .btn.q', '#start-regenerate'],
  ].map(([name, props, keys, mock, app]) => ({ name, props, keys, pixel: false,
    mock: { url: '?theme=light#2', selector: mock },
    app: { setup: "SIM.home.proposal=SIM.proposal('ready');", run: "document.getElementById('start-tab-review').click()", selector: app } })),
  { name: 'step-card-reaches-status-bar', app: {}, assert: "Math.round(document.getElementById('start-panel-workspace').getBoundingClientRect().bottom) === Math.round(document.querySelector('footer.statusbar').getBoundingClientRect().top)" },
  { name: 'workspace-footer-order', app: {}, assert: "[...document.querySelector('#start-panel-workspace .startup-actions').children].map(c => c.id).join() === 'start-save-note,start-discard,start-save-only,start-save'" },
  { name: 'roles-no-planning-row', app: { run: "document.getElementById('start-tab-team').click()" }, assert: "!document.querySelector('#start-roles .role-planning') && getComputedStyle(document.getElementById('start-discard')).display === 'none'" },
  { name: 'plan-edit-tools-wait-for-an-open-row', app: { setup: "SIM.home.proposal=SIM.proposal('ready');", run: "document.getElementById('start-tab-review').click()" }, assert: "['start-task-tools', 'start-edit', 'start-digest'].every(id => getComputedStyle(document.getElementById(id)).display === 'none') && getComputedStyle(document.querySelector('.plan-sources')).display === 'none'" },
  { name: 'plan-sources-row-is-not-in-the-mock', app: { setup: "SIM.home.proposal=SIM.proposal('ready');", run: "document.getElementById('start-tab-review').click(); document.querySelector('.startup-task')?.setAttribute('open', '')" }, assert: "!!document.querySelector('.startup-task[open]') && getComputedStyle(document.querySelector('.plan-sources')).display === 'none'" },
  { name: 'plan-sources-row-is-not-in-the-empty-plan', app: { setup: "SIM.home.proposal=null;", run: "document.getElementById('start-tab-review').click()" }, assert: "getComputedStyle(document.querySelector('.plan-sources')).display === 'none'" },
  // GitHub #684: a pending tool request is decided in the card footer (mock step 5).
  ...[
    ['tool-decision-note', ['fontSize', 'color'], ['fontSize', 'color'], '.card[data-show="g"] .actions .note', '#tool-decision-note'],
    ['tool-decision-deny', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor'], '.card[data-show="g"] .btn.q', '#tool-decision-deny'],
    ['tool-decision-allow', ['fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color'], ['height', 'fontSize', 'paddingTop', 'paddingLeft', 'borderTopLeftRadius', 'backgroundColor', 'color'], '.card[data-show="g"] .btn.p', '#tool-decision-allow'],
  ].map(([name, props, keys, mock, app]) => ({ name, props, keys, pixel: false,
    mock: { url: '?theme=light#4', selector: mock },
    app: { setup: STEPS_ENABLED + "SIM.room.tool_requests=[{id:'tr1',status:'pending',created_at:Math.floor(Date.now()/1000),context:{task:'task-1'},request:{tool:'Bash',input:{command:'npm install jsonwebtoken@9'}}}];", run: "document.querySelectorAll('#start-step-tabs [role=tab]')[3].click()", selector: app } })),
  // GitHub #727: the denied card's footer (mock step 4 denied = "h").
  ...[
    ['denied-footer-note', ['fontSize', 'color'], ['fontSize', 'color'], '.card[data-show="h"] .actions .note', '#denied-footer .note'],
    ['denied-footer-reassign', ['fontSize', 'paddingTop', 'borderTopLeftRadius'], ['height', 'fontSize', 'paddingTop', 'borderTopLeftRadius'], '.card[data-show="h"] .btn.q', '#denied-reassign'],
  ].map(([name, props, keys, mock, app]) => ({ name, props, keys, pixel: false,
    mock: { url: '?theme=light#5', selector: mock },
    app: { setup: STEPS_ENABLED + "SIM.room.report.tasks[0].status='permission_denied';SIM.room.report.tasks[0].error='permission denied: tool bash';", run: "document.querySelectorAll('#start-step-tabs [role=tab]')[3].click()", selector: app } })),
  { name: 'denied-footer-replaces-direction', assert: "(() => !document.getElementById('denied-footer').hidden && document.querySelector('#run-screen .composer').hidden)()",
    app: { setup: STEPS_ENABLED + "SIM.room.report.tasks[0].status='permission_denied';SIM.room.report.tasks[0].error='permission denied: tool bash';", run: "document.querySelectorAll('#start-step-tabs [role=tab]')[3].click()" } },
  { name: 'tool-decision-replaces-direction', assert: "(() => !document.getElementById('tool-decision').hidden && document.querySelector('#run-screen .composer').hidden && !document.querySelector('dialog[open]'))()",
    app: { setup: STEPS_ENABLED + "SIM.room.tool_requests=[{id:'tr1',status:'pending',created_at:Math.floor(Date.now()/1000),context:{task:'task-1'},request:{tool:'Bash',input:{command:'npm install jsonwebtoken@9'}}}];", run: "document.querySelectorAll('#start-step-tabs [role=tab]')[3].click()" } },
  { name: 'startup-inspector-closed', assert: "document.getElementById('start-inspector').getBoundingClientRect().width===0", app: {} },
  { name: 'startup-inspector-docked', assert: "(()=>{const r=document.querySelector('#start-inspector').getBoundingClientRect();return Math.round(r.right)===innerWidth&&r.top<200&&r.height>300})()",
    app: { run: "document.getElementById('start-inspector-toggle').click()" } },
];
