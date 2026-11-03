// GitHub #390 / R-210: one six-step stepper shared by the Workspace screen and the Agent Room.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const html=fs.readFileSync('src/ui/startup.html','utf8'),room=fs.readFileSync('src/ui/room.html','utf8'),roomJs=fs.readFileSync('src/ui/room.js','utf8');

// Markup: six tabs in order inside one tablist, grouped Prepare / Run / Finish.
const nav=html.match(/<nav id="start-step-tabs"[\s\S]*?<\/nav>/)[0];
const ids=[...nav.matchAll(/<button[^>]*id="start-tab-([a-z]+)"/g)].map(m=>m[1]);
assert.deepEqual(ids,['workspace','team','review','tasks','verify','merge']);
assert.deepEqual([...nav.matchAll(/data-group="([a-z]+)"/g)].map(m=>m[1]),['prepare','run','finish']);
for(const group of ['prepare','run','finish'])assert.match(nav,new RegExp(`data-group="${group}"[^>]*data-i18n-title="startup\\.group_${group}"`));
for(const name of ['tasks','verify','merge'])assert.doesNotMatch(nav.match(new RegExp(`<button[^>]*id="start-tab-${name}"[^>]*>`))[0],/aria-controls/,'steps 4-6 open the Room, they have no panel here');
assert.match(nav,/id="start-tab-tasks"[^>]*disabled/);
// The Room no longer carries its own five-stage strip.
assert.doesNotMatch(room,/data-stage|class="workflow"/);
assert.match(roomJs,/window\.acStepper\?\.paint\(stage,mergeRunning\)/);

// Behaviour of the stepper with a minimal DOM.
function node(id){const classes=new Set(),attrs={};return{id,attrs,tabIndex:0,disabled:false,classList:{toggle:(c,on)=>on?classes.add(c):classes.delete(c),has:c=>classes.has(c)},setAttribute(k,v){attrs[k]=v;}};}
const tabs=Object.fromEntries(ids.map(n=>[n,node('start-tab-'+n)]));
let inRoom=true;const document={getElementById:id=>tabs[id.replace('start-tab-','')],body:{classList:{contains:()=>inRoom}}};
const window={};vm.runInNewContext(fs.readFileSync('src/ui/stepper.js','utf8'),{window,document});
const s=window.acStepper,selected=()=>ids.filter(n=>tabs[n].attrs['aria-selected']==='true');
assert.equal(s.stepOf('plan'),'tasks');assert.equal(s.stepOf('task'),'tasks');assert.equal(s.stepOf('verify'),'verify');assert.equal(s.stepOf('merge'),'merge');assert.equal(s.stepOf('complete'),'merge');

s.paint('task');
assert.deepEqual(selected(),['tasks']);assert.equal(tabs.tasks.tabIndex,0);assert.equal(tabs.verify.tabIndex,-1);
assert.ok(['workspace','team','review'].every(n=>tabs[n].classList.has('done')),'steps 1-3 are done while a run is open');
assert.ok(!tabs.tasks.classList.has('done'));
s.paint('verify');assert.deepEqual(selected(),['verify']);assert.ok(tabs.tasks.classList.has('done'));
s.paint('merge',true);assert.deepEqual(selected(),['merge']);assert.ok(tabs.merge.classList.has('running'));assert.ok(tabs.verify.classList.has('done'));
s.paint('complete');assert.deepEqual(selected(),['merge']);assert.ok(!tabs.merge.classList.has('running'));
// A late Room snapshot must not re-mark the stepper after the operator went back to the Workspace.
inRoom=false;s.paint('verify');assert.deepEqual(selected(),['merge'],'unchanged while on the Workspace screen');inRoom=true;
// Leaving the Room clears the run marks and leaves steps 1-3 to showStep.
s.paint(null);assert.deepEqual(selected(),[]);assert.ok(ids.every(n=>!tabs[n].classList.has('done')&&!tabs[n].classList.has('running')));

s.available(false);assert.ok(['tasks','verify','merge'].every(n=>tabs[n].disabled));
s.available(true);assert.ok(['tasks','verify','merge'].every(n=>!tabs[n].disabled));

// Clicks: steps 4-6 open the run from the Workspace screen only; steps 1-3 leave the Room.
let listener;const navNode={addEventListener(type,fn){assert.equal(type,'click');listener=fn;}};
const calls=[];let onRun=false;
const doc2={getElementById:id=>id==='start-step-tabs'?navNode:tabs[id.replace('start-tab-','')],body:{classList:{contains:()=>onRun}}};
const win2={};vm.runInNewContext(fs.readFileSync('src/ui/stepper.js','utf8'),{window:win2,document:doc2});
win2.acStepper.bind({enter:()=>calls.push('enter'),home:()=>{calls.push('home');return Promise.resolve(true);},show:n=>calls.push('show:'+n)});
const click=name=>listener({target:{closest:()=>tabs[name]}});
click('verify');assert.deepEqual(calls,['enter']);
tabs.merge.disabled=true;click('merge');assert.deepEqual(calls,['enter'],'a disabled tab does nothing');tabs.merge.disabled=false;
click('team');assert.deepEqual(calls,['enter'],'on the Workspace screen showStep handles steps 1-3');
onRun=true;click('tasks');assert.deepEqual(calls,['enter'],'already in the Room');
click('team');
Promise.resolve().then(()=>Promise.resolve()).then(()=>{assert.deepEqual(calls,['enter','home','show:team']);console.log('Unified stepper: markup, run marks, availability and navigation passed');}).catch(e=>{console.error(e);process.exitCode=1;});
