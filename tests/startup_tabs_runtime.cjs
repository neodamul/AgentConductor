const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const markup=fs.readFileSync('src/ui/startup.html','utf8');
const nodes=new Map();let focused;
for(const tag of markup.matchAll(/<(?:button|section)\b[^>]*id="start-((?:tab|panel)-[a-z]+)"[^>]*>/g)){
 const attrs=Object.fromEntries([...tag[0].matchAll(/([\w-]+)="([^"]*)"/g)].map(m=>[m[1],m[2]]));
 nodes.set(tag[1],{attrs,hidden:/\shidden[\s>]/.test(tag[0]),tabIndex:Number(attrs.tabindex),setAttribute(k,v){this.attrs[k]=v;},focus(){focused=this;}});
}
const source=fs.readFileSync('src/ui/startup.js','utf8');
const code=source.slice(source.indexOf(' const stepNames='),source.indexOf(' const copy='));
nodes.set('save',{});
const context=vm.createContext({q:id=>nodes.get(id),mirrorProgress(){}});vm.runInContext(code,context);
const names=['workspace','team','review'];
function active(name){for(const n of names){const tab=nodes.get('tab-'+n),panel=nodes.get('panel-'+n),selected=n===name;assert.equal(tab.attrs.role,'tab');assert.equal(panel.attrs.role,'tabpanel');assert.equal(tab.attrs['aria-controls'],panel.attrs.id);assert.equal(panel.attrs['aria-labelledby'],tab.attrs.id);assert.equal(tab.attrs['aria-selected'],String(selected));assert.equal(tab.tabIndex,selected?0:-1);assert.equal(panel.hidden,!selected);}}
active('workspace');
// Input nodes stay attached to the same panels; switching is view-only.
const draft={value:'한글 draft goal',selectionStart:5};nodes.get('panel-workspace').input=draft;
nodes.get('tab-team').onclick();active('team');nodes.get('tab-review').onclick();active('review');nodes.get('tab-workspace').onclick();active('workspace');assert.equal(nodes.get('panel-workspace').input,draft);assert.equal(draft.value,'한글 draft goal');assert.equal(draft.selectionStart,5);
function key(name,key,next){let prevented=false;nodes.get('tab-'+name).onkeydown({key,preventDefault(){prevented=true;}});assert.ok(prevented);active(next);assert.equal(focused,nodes.get('tab-'+next));}
key('workspace','ArrowLeft','review');key('review','ArrowRight','workspace');key('workspace','End','review');key('review','Home','workspace');key('workspace','ArrowRight','team');
vm.runInContext("showStep('review')",context);active('review');vm.runInContext("showStep('invalid')",context);active('review');
console.log('Startup tabs: click, keyboard, accessibility and draft preservation passed');

// A manual Save advances only after persistence succeeds; internal saves do not route.
const saveStart=source.indexOf("q('save').onclick="),saveEnd=source.indexOf(";q('add')",saveStart);
vm.runInContext(source.slice(saveStart,saveEnd)+';',context);
(async()=>{
 let saved=true;const calls=[];context.save=async(...args)=>{calls.push(args);return saved;};
 await nodes.get('save').onclick({type:'click'});active('team');assert.equal(focused,nodes.get('tab-team'));assert.deepEqual(calls[0],[false]);
 vm.runInContext("showStep('workspace')",context);saved=false;
 await nodes.get('save').onclick({type:'click'});active('workspace');
 assert.match(markup,/<button id="start-save" class="primary">/);
 console.log('Workspace Save: green action, successful next-tab and failed-save retention passed');
})().catch(error=>{console.error(error);process.exitCode=1;});
