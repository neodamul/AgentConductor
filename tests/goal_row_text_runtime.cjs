// GitHub #753: a Goal row keeps the line breaks of its goal text, shows two lines and expands on demand.
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('goal row text skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-goal-row-text-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
const wait=ms=>new Promise(r=>setTimeout(r,ms)),fail=m=>{throw Error(m);};
const preset=`SIM.home.slugs=[{slug:'g',goal:'First line of the goal.\\nSecond line.\\nThird line.\\nFourth line.',active:true,plan:{phases:[]},progress:{stage:'plan',phases:[]}}];`;
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 const url=pathToFileURL(path.join(dir,'app.html')).href;await b.goto(url);await b.eval(`sessionStorage.setItem('SIM_PRESET',${JSON.stringify(preset)})`);await b.goto(url);await wait(1500);
 await b.eval("document.getElementById('start-goal-list-open').click()");await wait(800);
 const m=()=>b.eval("(()=>{const t=document.querySelector('.goal-row-text'),g=document.querySelector('.goal-row-toggle');if(!t||!g)return null;return {h:Math.round(t.getBoundingClientRect().height),open:t.classList.contains('open'),exp:g.getAttribute('aria-expanded'),label:g.textContent,ws:getComputedStyle(t).whiteSpace}})()");
 const closed=await m();if(!closed)fail('the Goal row needs its text block and toggle');
 if(closed.open||closed.exp!=='false'||!/펼치기|Expand/.test(closed.label)||closed.ws!=='pre-wrap')fail('collapsed state: '+JSON.stringify(closed));
 await b.eval("document.querySelector('.goal-row-toggle').click()");const open=await m();
 if(!open.open||open.exp!=='true'||!/접기|Collapse/.test(open.label)||open.h<=closed.h*1.5)fail('expanded state: '+JSON.stringify({closed,open}));
 await b.eval("document.querySelector('.goal-row-toggle').click()");const again=await m();
 if(again.open||again.h!==closed.h)fail('collapsing again: '+JSON.stringify(again));
 console.log('goal row text: collapsed to two lines with line breaks, expands and collapses');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
