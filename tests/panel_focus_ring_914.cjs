// #914: a focused step panel draws no focus ring even under WebKit's UA :focus rule (the mock has none).
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-panel-focus-914-'));let b;try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);b=await launch(findChrome(),{width:1280,height:900});
for(const theme of ['light','dark']){await b.goto(pathToFileURL(file).href);console.log(theme,await b.eval(`(async()=>{const q=id=>document.getElementById('start-'+id),wait=ms=>new Promise(r=>setTimeout(r,ms)),until=async(f,n)=>{for(let i=0;i<200;i++){if(f())return;await wait(25);}throw Error('timed out: '+n);},check=(n,ok)=>{if(!ok)throw Error(n);};document.documentElement.dataset.theme='${theme}';await until(()=>q('name').value,'workspace');
// WebKit draws its UA focus ring on any focused element; headless Chrome does not match :focus without window focus,
// so check that an author rule removes the outline of every focused step panel.
const rules=[...document.styleSheets].flatMap(sh=>[...sh.cssRules]).filter(r=>r.selectorText&&r.selectorText.includes(':focus')&&r.style.outlineStyle==='none');
const panels=[...document.querySelectorAll('[role="tabpanel"]')];check('panels exist',panels.length>=3);
for(const p of panels)check('no focus ring on '+(p.id||p.className),rules.some(r=>r.selectorText.split(',').some(sel=>sel.includes(':focus')&&!sel.includes('focus-visible')&&(()=>{try{return p.matches(sel.split(':focus').join(''))}catch(e){return false}})())));
q('jira-reference').click();await until(()=>q('source-reference').open,'reference dialog');q('source-reference').close();
// The Workspace body scrolls and clips outside its padding box: each field's focus ring must fit inside it.
const body=q('workspace-body'),inner=()=>{const r=body.getBoundingClientRect(),cs=getComputedStyle(body);return {left:r.left+parseFloat(cs.borderLeftWidth),right:r.left+parseFloat(cs.borderLeftWidth)+body.clientWidth};};
for(const id of ['name','repository','goal']){const e=q(id),ring=4/* widest field ring in room.css: 2px outline + 2px offset */,r=e.getBoundingClientRect(),box=inner();check(id+' focus ring fits the scroll box ('+[r.left-ring,box.left,r.right+ring,box.right].map(Math.round)+')',ring>0&&r.left-ring>=box.left-0.5&&r.right+ring<=box.right+0.5);}
return 'panel focus checks passed';})()`));}
}finally{if(b)await b.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
