// #916/#917: operate the real assembled UI; no agent or external MCP server runs.
const assert=require('node:assert/strict'),fs=require('node:fs'),os=require('node:os'),path=require('node:path');
const {execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
const specs=require('./design_parity/source_review_spec.cjs');
const fixture=done=>specs.find(s=>s.name==='source-review-'+(done?'summary':'card')).app.setup;
(async()=>{
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-source-review-916-'));let browser;
 try{
  const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
  browser=await launch(findChrome(),{width:1280,height:900});
  for(const theme of ['light','dark'])for(const width of [840,1280]){
   await browser.viewport(width,900);await browser.goto(pathToFileURL(file).href);
   const setup=fixture(process.argv.includes('--layout'));
   await browser.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(setup)})`);await browser.goto(pathToFileURL(file).href);
   const result=await browser.eval(`(async()=>{
    const q=id=>document.getElementById('start-'+id),until=async(name,f)=>{const end=Date.now()+12000;while(!f()){if(Date.now()>end)throw Error(name);await wait(25);}},check=(name,ok)=>{if(!ok)throw Error(name);};
    document.documentElement.dataset.theme='${theme}';
    await until('source cards ready',()=>q('mcp-records').querySelector('.source-doc'));
    if(!q('mcp-review').open)q('mcp-review').showModal();
    if(!${process.argv.includes('--layout')}){
     check('guide initially expanded',q('mcp-guide').open);q('mcp-guide').querySelector('summary').click();
     SIM.home.mcp_progress.documents[1].title='Updated after collapse';
     await until('progress refresh',()=>q('mcp-records').textContent.includes('Updated after collapse'));
     check('manual collapse survives progress update',!q('mcp-guide').open);
     q('mcp-guide').querySelector('summary').click();check('operator can expand again',q('mcp-guide').open);
     ${fixture(true)}
     await until('completed sources ready',()=>q('mcp-records').querySelector('.source-summary'));
     check('manual expansion survives completion',q('mcp-guide').open);
    }
    const long='LONG_REFERENCE_'.repeat(20)+'-1',r=SIM.home.mcp_result;
    r.records[0].reference.id=long;r.records[0].title='A long source title '.repeat(40);
    r.failures=[];SIM.home.mcp_progress.documents=SIM.home.mcp_progress.documents.slice(0,2);
    SIM.home.mcp_progress.documents[0].reference='jira:'+long;SIM.home.mcp_progress.documents[0].title=r.records[0].title;
    SIM.home.job.id='long-headers';
    await until('long headers rendered',()=>q('mcp-records').textContent.includes(long));
    const dialog=q('mcp-review'),checks=[...dialog.querySelectorAll('.source-doc-head input')];
    check('two selected records',checks.length===2&&checks.every(n=>n.checked));
    for(const card of dialog.querySelectorAll('.source-doc')){
     const head=card.querySelector('.source-doc-head'),box=head.getBoundingClientRect();
     for(const node of head.children){const b=node.getBoundingClientRect();check('header child stays inside card: '+node.className,b.left>=box.left-1&&b.right<=box.right+1);}
    }
    check('no horizontal dialog overflow',dialog.scrollWidth<=dialog.clientWidth+1);
    check('checkbox has bounded geometry',checks.every(n=>{const b=n.getBoundingClientRect();return b.width===16&&b.height===16;}));
    check('required source is visibly checked',checks[0].disabled&&getComputedStyle(checks[0],'::after').content==='""'&&getComputedStyle(checks[0]).opacity==='1');
    check('checkboxes named',checks.every(n=>n.getAttribute('aria-label')===n.value));
    const ref=dialog.querySelector('.source-ref');check('full reference tooltip',ref.title===ref.textContent);
    checks[1].click();check('optional source can be unchecked',!checks[1].checked);
    return 'guide persistence, bounded headers and source selection passed';
   })()`);console.log(theme,width,result);
   if(width===840)assert(await browser.eval("document.getElementById('start-mcp-review').open"));
   await browser.eval(`(async()=>{const q=id=>document.getElementById('start-'+id),expected=q('mcp-records').querySelector('input:checked').value;SIM.reply.mcp_apply=p=>{SIM.applied=p.selected;SIM.home.imports={selected:p.selected};return {applied:true};};q('mcp-apply').click();const end=Date.now()+12000;while(!SIM.applied||q('mcp-review').open){if(Date.now()>end)throw Error('Apply did not finish');await wait(25);}if(SIM.applied.length!==1||SIM.applied[0]!==expected)throw Error('only selected reference is applied');})()`);
  }
 }finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}
})().catch(e=>{console.error(e);process.exitCode=1;});
