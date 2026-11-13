// #948: long verification commands stay compact without hiding the full editable value.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const chrome=findChrome();if(!chrome)throw Error('Chrome required');const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-plan-row-948-'));let b;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(chrome,{width:1280,height:960});
const command=['sh','-c',Array(20).fill('test -f docs/check.md && grep -q marker docs/check.md').join(' && ')];
const preset=`SIM.home.proposal=SIM.proposal('ready');SIM.home.proposal.plan.tasks[0].test_command=${JSON.stringify(command)};SIM.home.proposal.plan.tasks[0].paths=['docs/'+ 'nested/'.repeat(30)+'check.md'];`;
for(const width of [840,1280])for(const theme of ['light','dark']){
 await b.viewport(width,960);await b.goto(pathToFileURL(file).href);await b.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(preset)})`);await b.goto(pathToFileURL(file).href);
 await b.eval(`(async()=>{const end=Date.now()+6000;while(!document.querySelector('#start-tasks .task-check')){if(Date.now()>end)throw Error('Plan did not render');await new Promise(r=>setTimeout(r,30));}document.getElementById('start-tab-review').click();document.documentElement.dataset.theme='${theme}';})()`);
 const result=await b.eval(`(()=>{const rows=[...document.querySelectorAll('#start-tasks .startup-task')],summary=rows[0].querySelector('summary'),check=summary.querySelector('.task-check'),owner=summary.querySelector('.task-owner'),style=getComputedStyle(check);if(summary.getBoundingClientRect().height>40)throw Error('Collapsed row wraps: '+summary.getBoundingClientRect().height);if(style.whiteSpace!=='nowrap'||style.textOverflow!=='ellipsis'||check.scrollWidth<=check.clientWidth)throw Error('Long command is not truncated');if(check.title!==check.textContent||owner.title!==owner.textContent)throw Error('Full tooltip missing');rows[0].open=true;const edit=rows[0].querySelector('[data-key=test_command]');if(JSON.stringify(JSON.parse(edit.value))!==${JSON.stringify(JSON.stringify(command))})throw Error('Full command lost in editor');return 'compact row, tooltip, expanded command';})()`);console.log(theme,width,result);
}
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
