// GitHub #226 / #490: an unused placeholder workspace is never shown; the page asks for a name instead, and the question
// can be cancelled (button or Escape): nothing is created and it is asked again on the next page load.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const source=fs.readFileSync('src/ui/startup.js','utf8'),html=fs.readFileSync('src/ui/startup.html','utf8');
assert(html.includes('id="start-new-cancel"'),'the dialog has a cancel button');
const nodes=new Map(),shown=[];
function q(id){if(!nodes.has(id))nodes.set(id,{value:'x',open:false,hidden:false,showModal(){this.open=true;shown.push(id);}});return nodes.get(id);}
const context={q,acMessage:key=>key,AC_I18N:{bind(node,fn){node.text=fn();}},state:null};
vm.createContext(context);
const a=source.indexOf('let firstWorkspaceAsked'),b=source.indexOf('/*first-workspace-end*/');
assert(a>=0&&b>a);
vm.runInContext(source.slice(a,b),context);
const ask=()=>vm.runInContext('askFirstWorkspace()',context);
context.state={workspace_blank:false};ask();
assert.equal(shown.length,0,'a real workspace is not interrupted');
context.state={workspace_blank:true};ask();
assert.deepEqual(shown,['new-workspace']);
assert.equal(q('new-name').value,'');
assert.equal(q('new-feedback').text,'startup.add_a_workspace_to_start');
assert.notEqual(q('new-cancel').hidden,true,'the first workspace can be cancelled');
assert.equal(q('new-workspace').oncancel,undefined,'Escape closes it');
q('new-workspace').open=false;ask();
assert.equal(shown.length,1,'asked once per page load');
assert(!source.includes('firstWorkspaceNamed'),'no state to restore after naming');
assert(source.includes('askFirstWorkspace();optimize.update();sources.update();chat.update();'),'the poll render asks');
console.log('A blank placeholder workspace makes the page ask for a first workspace name, which can be cancelled.');
