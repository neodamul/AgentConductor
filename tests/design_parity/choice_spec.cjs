// Intentional exception (design owner, #409): planning is informational; Workspace owns
// coordinator/model. Five backend worker roles stay unchanged; follow-up #429/#430.
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
 const atom=`const r=b.getBoundingClientRect();b.style.position='fixed';b.style.left='24px';b.style.top='120px';b.style.width='160px';b.style.flex='none';b.style.zIndex='99';`;
 entries.push({name:`role-disabled-model-${theme}-${width}`,viewport:{width,height:960},props,maxPixelDiff:.02,
  mock:{url:`?theme=${theme}#1`,selector:'[data-assignment-target="role-0"][data-assignment-field="fallbackModel"]',run:`const b=document.querySelector('[data-assignment-target="role-0"][data-assignment-field="fallbackModel"]');`+atom},
  app:{selector:'#start-role-development-fallback-model + .ac-choice-trigger',run:`document.documentElement.dataset.theme='${theme}';document.getElementById('start-tab-team').click();const s=document.getElementById('start-role-development-fallback');s.value='';s.dispatchEvent(new Event('change',{bubbles:true}));acChoices.sync();const b=acChoices.buttonFor(document.getElementById('start-role-development-fallback-model'));`+atom}});
 const arrowAtom=`b.style.position='fixed';b.style.left='24px';b.style.top='120px';b.style.zIndex='99';b.style.background='var(--surface)';`;
 entries.push({name:`role-fallback-arrow-${theme}-${width}`,viewport:{width,height:960},props:['color'],maxPixelDiff:.02,
  mock:{url:`?theme=${theme}#1`,selector:'.assignment-fallback-arrow svg',run:`const b=document.querySelector('.assignment-fallback-arrow svg');`+arrowAtom},
  app:{selector:'.role-fallback-arrow svg',run:`document.documentElement.dataset.theme='${theme}';document.getElementById('start-tab-team').click();const b=document.querySelector('.role-fallback-arrow svg');`+arrowAtom}});

 const manualOrigin=`const fr=f.getBoundingClientRect();f.style.position='relative';f.style.left=(Math.round(fr.left)-fr.left)+'px';f.style.top=(Math.round(fr.top)-fr.top)+'px';`;
 entries.push({name:`choice-manual-form-${theme}-${width}`,viewport:{width,height:960},props,maxPixelDiff:.02,
  mock:{url:`?theme=${theme}#0`,selector:'#assignment-manual-form',run:mock+align+`openSelectionMenu({anchor:b,title:'모델',value:'',options:[],onSelect:()=>{}});const f=document.getElementById('assignment-manual-form');f.hidden=false;`+manualOrigin},
  app:{selector:'#ac-choice-menu form',run:setup+align+`const s=document.getElementById('start-coordinator-model');s.dataset.choiceManual='true';acChoices.open(s);document.querySelector('#ac-choice-menu [data-value=__manual]').click();const f=document.querySelector('#ac-choice-menu form');document.activeElement.blur();`+manualOrigin}});

}
module.exports=entries;
