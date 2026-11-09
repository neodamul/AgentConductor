// GitHub #663: a quota refresh updates the quota ring only; the card's other buttons stay the same DOM nodes (no flicker).
const fs=require('fs'),path=require('path'),os=require('os'),{execFileSync}=require('child_process'),{pathToFileURL}=require('url');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('quota ring isolation skipped: no Chrome');process.exit(77);}
const root=path.resolve(__dirname,'..'),dir=fs.mkdtempSync(path.join(os.tmpdir(),'ac-quota-ring-'));
execFileSync('node',[path.join(root,'tests/ui_sim/build.cjs'),root,path.join(dir,'app.html')]);
(async()=>{try{const b=await launch(chrome,{width:1280,height:900});try{
 const url=pathToFileURL(path.join(dir,'app.html')).href;await b.goto(url);
 let ready=false;for(let i=0;i<100;i++){try{ready=await b.eval("document.querySelectorAll('.startup-profile .quota-controls .usage-button').length>0");}catch{}if(ready)break;await new Promise(r=>setTimeout(r,50));}
 if(!ready)throw Error('agent cards with quota rings did not render');
 await b.eval("window.__tools=[...document.querySelectorAll('.startup-profile .profile-tools button, .startup-profile .agent-symbol, .startup-profile .name-row strong')];window.__ring=document.querySelector('.startup-profile .quota-controls .usage-button');window.__title=window.__ring.title;SIM.home.quota_indicators.claude.rings[0].remaining_percent=7;SIM.home.quota_indicators.claude.rings[0].used_percent=93;SIM.home.quota_indicators.claude.rings[0].label=SIM.home.quota_indicators.claude.rings[0].label;true");
 let result=null;for(let i=0;i<160;i++){await new Promise(r=>setTimeout(r,100));result=await b.eval("({same:window.__tools.every(n=>n.isConnected),count:window.__tools.length,ringChanged:document.querySelector('.startup-profile .quota-controls .usage-button').title!==window.__title,ring:document.querySelector('.startup-profile .quota-controls .usage-button').title})");if(result.ringChanged)break;}
 if(!result.ringChanged)throw Error('the quota ring did not pick up the refreshed value: '+JSON.stringify(result));
 if(!result.same||!result.count)throw Error('a quota refresh rebuilt the card controls: '+JSON.stringify(result));
 console.log('quota refresh updated the ring only; card controls kept',result.count,'nodes');
}finally{await b.close();}}finally{fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
