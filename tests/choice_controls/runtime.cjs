// #409: exercise the real assembled webview, including native-modal focus scope.
const fs=require('fs'),path=require('path'),os=require('os'),assert=require('assert');
const {execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('../design_parity/cdp.cjs');
const chrome=findChrome();if(!chrome){console.log('choice controls skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'../..'),out=fs.mkdtempSync(path.join(os.tmpdir(),'ac-choice-test-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(out,'app.html')]);
(async()=>{try{
 for(const width of [840,1280]){
  const browser=await launch(chrome,{width,height:960,scale:2});
  try{for(const theme of ['light','dark']){
   await browser.goto(pathToFileURL(path.join(out,'app.html')).href+'?case='+width+theme);
   const result=await browser.eval(`(async()=>{
    const checks=[],check=(name,value)=>{if(!value)throw Error(name);checks.push(name);};
    const settle=()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
    document.documentElement.dataset.theme=${JSON.stringify(theme)};await settle();
    check('shared choice surface exists',!!window.acChoices);
    const select=document.getElementById('start-coordinator'),button=acChoices.buttonFor(select);
    check('native source retained and hidden',select.hidden&&!!button);
    check('provider display name',button.textContent.includes('Claude Code'));
    const before=select.value;button.click();await settle();
    const menu=document.getElementById('ac-choice-menu');
    check('list opens at field',!menu.hidden&&button.getAttribute('aria-expanded')==='true');select.setAttribute('aria-label','Updated coordinator label');await settle();check('accessibility label refresh keeps choice list open',!menu.hidden);
    const a=button.getBoundingClientRect(),m=menu.getBoundingClientRect();
    check('anchored six pixel gap',Math.abs(m.top-a.bottom-6)<.6||Math.abs(a.top-m.bottom-6)<.6);
    check('list stays in app viewport',m.left>=8&&m.right<=innerWidth-8&&m.top>=8&&m.bottom<=innerHeight-8);
    check('current choice has aria selection',!!menu.querySelector('[aria-selected="true"]'));
    document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));
    check('escape preserves value and focus',menu.hidden&&select.value===before&&document.activeElement===button);
    button.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true,cancelable:true}));
    check('keyboard opens list',!menu.hidden);
    const items=[...menu.querySelectorAll('[role="option"]:not(:disabled)')];
    const target=items.find(e=>e.dataset.value==='codex');target.focus();target.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,cancelable:true}));await settle();
    check('keyboard selects immediately',select.value==='codex'&&menu.hidden&&document.activeElement===button);
    button.click();document.body.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));
    check('outside dismiss restores focus',menu.hidden&&document.activeElement===button);
    const dialog=document.createElement('dialog'),dynamic=document.createElement('select');
    dynamic.setAttribute('aria-label','Modal fixture');dynamic.innerHTML='<option value="a">Alpha</option><option value="b">Beta</option>';
    dialog.append(dynamic);document.body.append(dialog);dialog.showModal();await settle();
    const db=acChoices.buttonFor(dynamic);check('dynamic modal fields enhanced',!!db);db.click();
    check('list belongs to active modal',menu.parentElement===dialog);
    document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));
    check('escape closes list only',menu.hidden&&dialog.open&&document.activeElement===db);
    dynamic.disabled=true;await settle();check('disabled state synchronized',db.disabled);
    dynamic.disabled=false;dynamic.options[0].textContent='Updated';await settle();check('option updates synchronized',db.textContent.includes('Updated'));
    dynamic.dataset.choiceManual='true';let manualCalls=0;
    dynamic.acManualPick=async value=>{manualCalls++;if(value==='reject')throw Error('fixture rejection');};
    const manual=()=>{db.click();menu.querySelector('[data-value="__manual"]').click();return menu.querySelector('input');};
    let entry=manual();check('manual controls are translated',menu.querySelector('label').textContent===acMessage('choices.model_id')&&menu.querySelector('form button').textContent===acMessage('choices.apply'));entry.value='-invalid';menu.querySelector('form').dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));await settle();
    check('invalid manual model never commits',manualCalls===0&&!menu.hidden&&dynamic.value==='a');
    entry.value='configured-model';entry.dispatchEvent(new Event('input'));menu.querySelector('form').dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));await settle();
    check('explicit manual registration commits after success',manualCalls===1&&menu.hidden&&dynamic.value==='configured-model');
    entry=manual();entry.value='reject';menu.querySelector('form').dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}));await settle();
    check('registration error preserves selection and popup',!menu.hidden&&dynamic.value==='configured-model'&&menu.querySelector('[role="alert"]').textContent==='fixture rejection');
    acChoices.close(true);
    dialog.close();dialog.remove();await settle();check('single shared surface',document.querySelectorAll('#ac-choice-menu').length===1);
    return checks;
   })()`);
   console.log(theme,width,result.length,'choice checks passed');assert(result.length>=15);
  }}finally{await browser.close();}
 }
}finally{fs.rmSync(out,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
