// #409: compare field/list atoms at a fixed width and pixel-aligned fixture origin.
// Shell geometry is #408; fractional parent placement must not change text rasterization.
const props=['borderTopWidth','borderTopLeftRadius','paddingTop','paddingRight','paddingBottom','paddingLeft','backgroundColor','color','fontSize','fontFamily','fontWeight','lineHeight','letterSpacing'];
const entries=[];
for(const width of [840,1280])for(const theme of ['light','dark']){
 const setup=`document.documentElement.dataset.theme='${theme}';const b=acChoices.buttonFor(document.getElementById('start-coordinator'));b.style.width='160px';b.style.flex='none';b.parentElement.style.width='160px';b.parentElement.style.flex='none';document.getElementById('start-coordinator').setAttribute('aria-label','코디네이터 에이전트');acChoices.sync();`;
 const align=`const r=b.getBoundingClientRect();b.style.position='relative';b.style.left=(Math.round(r.left)-r.left)+'px';b.style.top=(Math.round(r.top)-r.top)+'px';`;
 const mock=`workspaceCoordinator.agent='claude';refreshAssignmentChoices();const b=document.querySelector('[data-assignment-target="coordinator"][data-assignment-field="agent"]');b.style.width='160px';b.style.flex='none';`;
 entries.push({name:`choice-field-${theme}-${width}`,viewport:{width,height:960},props,maxPixelDiff:.02,mock:{url:`?theme=${theme}#0`,selector:'[data-assignment-target="coordinator"][data-assignment-field="agent"]',run:mock+align},app:{selector:'#start-coordinator + .ac-choice-trigger',run:setup+align}});
 entries.push({name:`choice-list-${theme}-${width}`,viewport:{width,height:960},props,maxPixelDiff:.02,mock:{url:`?theme=${theme}#0`,selector:'#assignment-menu',run:mock+align+`assignmentCatalog.claude=['opus'];openSelectionMenu({anchor:b,title:'코디네이터 에이전트',value:'claude',options:[{value:'claude',label:'Claude Code'},{value:'codex',label:'Codex'}],onSelect:()=>{}});`},app:{selector:'#ac-choice-menu',run:setup+align+`b.click();`}});
}
module.exports=entries;
