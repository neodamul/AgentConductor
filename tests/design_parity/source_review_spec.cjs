// #903: the document preview window's guide and per-document cards match the approved mock while running and on review.
const progress = (final) => `(()=>{const t0=Date.now()-239000,log=(at,event,detail='')=>({at_ms:t0+at,event,detail});
SIM.home.config.goal='[Jira:EMM-52599] [Confluence:53553565] [Jira:EMM-52611]';
SIM.home.job={id:'j903',kind:'mcp',state:${final ? "'completed'" : "'running'"},started_ms:t0};
SIM.home.mcp_progress={started_ms:t0,documents:[{reference:'jira:EMM-52599',stage:${final ? "'done'" : "'summary_wait'"},title:'Retry policy',started_ms:t0,ended_ms:t0+112000,log:[log(0,'coordinator','claude'),log(38000,'selected','atlassian · get · 1/8'),log(40000,'mcp_call','300'),log(112000,'read_done','34816')]},
{reference:'confluence:53553565',stage:${final ? "'done'" : "'reading'"},title:'Architecture',started_ms:t0+112000,${final ? 'ended_ms:t0+340000,' : ''}log:[log(112000,'coordinator','claude'),log(153000,'selected','atlassian · get · 1/8'),log(155000,'mcp_call','300')]},
${final ? "{reference:'jira:EMM-52611',stage:'failed',started_ms:t0+340000,ended_ms:t0+640000,error:'timed out',log:[log(340000,'coordinator','claude'),log(383000,'mcp_call','300'),log(640000,'failed')]}" : "{reference:'jira:EMM-52611',stage:'waiting',log:[]}"}]};
${final ? "const rec=(kind,id,title)=>({reference:{kind,id},server:'atlassian',tool:'get',title,url:null,version:'7',updated_at:null,sha256:'abcdef0123456789',body:'x'.repeat(34816)});SIM.home.mcp_result={id:'p903',goal_sha256:'g',settings_sha256:'s',fetched_at:Math.floor(t0/1000),records:[rec('jira','EMM-52599','Retry policy'),rec('confluence','53553565','Architecture')],failures:[{reference:{kind:'jira',id:'EMM-52611'},error:'timed out'}],summaries:{'jira:EMM-52599':{text:'Summary one'},'confluence:53553565':{text:'Summary two'}}};" : ''}})();`;
const OPEN = "(()=>{if(!document.querySelector('#start-mcp-records .source-doc'))throw Error('cards not rendered');const d=document.getElementById('start-mcp-review');if(!d.open)d.showModal();})()";
const REVIEW = "(()=>{if(!document.querySelector('#start-mcp-records .source-summary'))throw Error('review not rendered');})()";
const TEXT = ['fontSize', 'fontWeight', 'lineHeight', 'color'];
const BOXED = ['backgroundColor', 'borderTopWidth', 'borderTopColor', 'borderTopLeftRadius', 'paddingTop', 'paddingLeft'];
const entry = (name, props, mockSel, appSel, final = false, index = 0) => ({ name: 'source-review-' + name, props, keys: props, pixel: false,
  mock: { url: `?theme=light&sources=${final ? 'review' : 'running'}#0`, selector: '#source-review ' + mockSel, index },
  app: { setup: progress(final), run: final ? REVIEW : OPEN, selector: '#start-mcp-review ' + appSel, index } });
module.exports = [
  entry('guide', [...BOXED, ...TEXT, 'marginBottom'], '.source-guide', '.source-guide'),
  entry('guide-note', [...TEXT, 'marginTop'], '.source-guide p', '.source-guide p'),
  entry('card', [...BOXED, 'marginBottom', 'boxShadow'], '.source-doc', '.source-doc'),
  entry('card-running', ['borderTopColor', 'boxShadow'], '.source-doc.running', '.source-doc.running'),
  entry('ref-tag', [...BOXED, ...TEXT], '.source-ref', '.source-ref'),
  entry('state-ready', [...BOXED, ...TEXT], '.source-state.ready', '.source-state.ready'),
  entry('state-running', ['backgroundColor', 'color'], '.source-state.running', '.source-state.running'),
  entry('state-waiting', ['backgroundColor', 'color'], '.source-state.waiting', '.source-state.waiting'),
  entry('title', [...TEXT, 'textOverflow'], '.source-title', '.source-title'),
  entry('title-pending', ['fontWeight', 'color'], '.source-title.pending', '.source-title.pending'),
  entry('time', [...TEXT, 'fontVariantNumeric'], '.source-time', '.source-time'),
  entry('step-done', [...TEXT, 'borderTopWidth', 'borderTopColor', 'paddingTop'], '.source-step.done', '.source-step.done'),
  entry('step-current', [...TEXT, 'borderTopColor'], '.source-step.current', '.source-step.current'),
  entry('step-idle', [...TEXT, 'borderTopColor'], '.source-step:not(.done):not(.current)', '.source-step:not(.done):not(.current)'),
  entry('log', [...BOXED, 'fontFamily', 'fontSize', 'lineHeight', 'color', 'whiteSpace', 'marginTop'], '.source-log', '.source-log'),
  entry('meta', [...TEXT], '.source-meta', '.source-meta'),
  entry('foot', ['display', 'justifyContent', 'alignItems', 'columnGap', 'borderTopWidth', 'borderTopColor', 'paddingTop'], '.source-foot', '.source-foot'),
  entry('summary', [...BOXED, ...TEXT, 'marginTop'], '.source-summary', '.source-summary', true),
  entry('full-source', [...TEXT, 'marginTop'], '.source-full>summary', '.source-full>summary', true),
  entry('provenance', [...TEXT, 'marginTop'], '.source-provenance', '.source-provenance', true),
  entry('step-failed', [...TEXT, 'borderTopColor'], '.source-step.failed', '.source-step.failed', true),
  entry('state-failed', ['backgroundColor', 'color'], '.source-state.failed', '.source-state.failed', true),
  entry('error', [...TEXT, 'marginTop'], '.source-error', '.source-error', true),
];
