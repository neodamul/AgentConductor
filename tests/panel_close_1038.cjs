// GitHub #1038: slide panels close on a press outside them; the coordinator chat also closes from a bottom button.
const fs=require('node:fs'),os=require('node:os'),path=require('node:path'),{execFileSync}=require('node:child_process'),{pathToFileURL}=require('node:url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');
(async()=>{const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-panel-close-'));let browser;
try{const file=path.join(dir,'app.html');execFileSync(process.execPath,['tests/ui_sim/build.cjs',process.cwd(),file]);
browser=await launch(findChrome(),{width:1280});
for(const theme of ['light','dark']){await browser.goto(pathToFileURL(file).href);
const out=await browser.eval(`(async()=>{document.documentElement.dataset.theme='${theme}';const q=id=>document.getElementById('start-'+id),check=(n,ok)=>{if(!ok)throw Error(n);return n;},w=ms=>new Promise(r=>setTimeout(r,ms)),done=[];
const until=async(f,n)=>{const end=Date.now()+5000;while(!f()){if(Date.now()>end)throw Error(n);await w(30);}};
const outside=d=>d.dispatchEvent(new MouseEvent('click',{bubbles:true,clientX:5,clientY:300}));
q('settings-button').click();await until(()=>q('settings').open,'settings opens');q('settings').dispatchEvent(new MouseEvent('click',{bubbles:true,clientX:innerWidth-20,clientY:300}));done.push(check('a click inside Settings keeps it open',q('settings').open));
outside(q('settings'));await until(()=>!q('settings').open,'settings closes');done.push(check('a click outside Settings closes it without saving',q('settings').returnValue==='cancel'&&!SIM.log.some(e=>e.kind==='set_language')));
q('goal-list-open').click();await until(()=>q('goals').open,'goal list opens');outside(q('goals'));await until(()=>!q('goals').open,'goal list closes');done.push(check('a click outside the Goal list closes it',!q('goals').open));
q('chat-button').click();await until(()=>!q('chat').hidden,'chat opens');q('chat-input').dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));done.push(check('a press inside the chat keeps it open',!q('chat').hidden));
document.querySelector('main,#startup-screen').dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));await until(()=>q('chat').hidden,'chat closes');done.push(check('a press outside the chat closes it',q('chat').hidden));
q('chat-button').click();await until(()=>!q('chat').hidden,'chat reopens');const close=q('chat-close-bottom'),send=q('chat-send');
done.push(check('the chat has a bottom Close button beside Send',close.textContent.trim()===acMessage('startup.close')&&close.nextElementSibling===send&&close.getBoundingClientRect().top>=q('chat-input').getBoundingClientRect().bottom));
close.click();await until(()=>q('chat').hidden,'bottom close');done.push(check('the bottom Close button closes the chat',q('chat').hidden));return done;})()`);
console.log(theme,out.length,'panel close checks passed');}
}finally{if(browser)await browser.close();fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
