// #907: the reference count, the over-limit error and the disabled add buttons match the approved mock.
const goal = n => ['[Jira:A-1]', '[Jira:A-2]', '[Confluence:3]', '[Confluence:4]', '[Jira:A-5]', '[Jira:A-6]'].slice(0, n).join(' ');
const mockRun = n => `(()=>{i=0;show();const g=document.getElementById('goal-entry');g.value=${JSON.stringify(goal(n))};g.dispatchEvent(new Event('input'));})()`;
const appSetup = n => `SIM.home.config.goal=${JSON.stringify(goal(n))};SIM.reply.mcp_source_suggestions=p=>{const refs=[...new Set([...p.goal.matchAll(/\\[(Jira|Confluence):([^\\]]+)\\]/g)].map(m=>m[1].toLowerCase()+':'+m[2]))];return {references:refs,reference_ranges:[],suggestions:[]};};`;
const appRun = n => `(()=>{if(!document.getElementById('start-source-status').textContent.includes(${JSON.stringify(n > 5 ? '(' : n + '/5')}))throw Error('status not ready');})()`;
const TEXT = ['fontSize', 'fontWeight', 'lineHeight', 'color'];
const entry = (name, props, n, mockSel, appSel) => ({ name: 'reference-limit-' + name, props, keys: props, pixel: false,
  mock: { url: '?theme=light#0', run: mockRun(n), selector: mockSel }, app: { setup: appSetup(n), run: appRun(n), selector: appSel } });
module.exports = [
  entry('count', TEXT, 4, '#source-status', '#start-source-status'),
  entry('over', TEXT, 6, '#source-status.over', '#start-source-status.over'),
  entry('add-disabled', ['opacity', 'cursor'], 5, '#source-jira', '#start-jira-reference'),
];
