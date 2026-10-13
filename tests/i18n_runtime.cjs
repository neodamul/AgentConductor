// ID localization: language changes affect only marked UI content.
const assert=require('node:assert/strict');
const fs=require('node:fs'),vm=require('node:vm');
const en=JSON.parse(fs.readFileSync('src/ui/messages/en.json','utf8'));
const ko=JSON.parse(fs.readFileSync('src/ui/messages/ko.json','utf8'));
class Node {
  constructor(tag='SPAN',attrs={}){this.nodeType=1;this.tagName=tag;this.attrs={...attrs};this.childNodes=[];this.value='';this._text='';}
  getAttribute(key){return this.attrs[key]??null;}
  setAttribute(key,value){this.attrs[key]=String(value);}
  removeAttribute(key){delete this.attrs[key];}
  append(...nodes){this.childNodes.push(...nodes);}
  replaceChildren(...nodes){this.childNodes=[...nodes];this._text='';}
  get textContent(){return this.childNodes.length?this.childNodes.map(n=>n.textContent).join(''):this._text;}
  set textContent(value){this.childNodes=[];this._text=String(value);}
}
const fixed=new Node('SPAN',{'data-i18n':'startup.workspace_settings'});
const custom=new Node();custom.textContent='Workspace settings';
const goal=new Node('TEXTAREA',{'data-i18n-placeholder':'startup.describe_the_behavior_constraints_and_expected_result'});
goal.value='Workspace settings \n Goal {value1} <b>Keep me</b>';
const raw=new Node('PRE');raw.textContent='Workspace settings\nNative bridge unavailable';
const input=new Node('INPUT',{'data-i18n-title':'startup.drag_to_reorder'});input.value='Agent';
const dynamic=new Node();
const body=new Node('BODY');body.append(fixed,custom,goal,raw,input,dynamic);
const listeners=new Map(),root=new Node('HTML');root.lang='en';
const document={body,documentElement:root,createTextNode(text){return {nodeType:3,childNodes:[],textContent:text};},dispatchEvent(e){for(const fn of listeners.get(e.type)||[])fn(e);},addEventListener(name,fn){const list=listeners.get(name)||[];list.push(fn);listeners.set(name,list);}};
const context={document,Event:class Event{constructor(type){this.type=type;}},AC_MESSAGES:{en,ko}};
const source=fs.readFileSync('src/ui/i18n.js','utf8').split('// DD-111:')[0];
const {AC_I18N:i18n,acMessage:m}=vm.runInNewContext(source+';({AC_I18N,acMessage})',context);
assert.equal(custom.textContent,'Workspace settings');
i18n.bind(dynamic,()=>m('startup.refresh_quota',{value1:'my agent {value2}'}));
const sample=document.createTextNode('');i18n.text(sample,()=>m('room.completed_missing'));
assert.equal(sample.textContent,'room.completed_missing');
assert.equal(i18n.format('constructor'),'constructor');
assert.equal(dynamic.textContent,'Refresh my agent {value2} quota');
let switches=0;document.addEventListener('ac-language-change',()=>switches++);
i18n.set('ko');
assert.equal(root.lang,'ko');assert.equal(fixed.textContent,'워크스페이스 설정');
assert.equal(goal.getAttribute('placeholder'),ko['startup.describe_the_behavior_constraints_and_expected_result']);
assert.equal(input.getAttribute('title'),'드래그해서 순서 변경');
assert.equal(dynamic.textContent,'my agent {value2} 사용량 새로고침');
assert.equal(custom.textContent,'Workspace settings');assert.equal(raw.textContent,'Workspace settings\nNative bridge unavailable');
assert.equal(goal.value,'Workspace settings \n Goal {value1} <b>Keep me</b>');assert.equal(input.value,'Agent');
const original=ko['startup.refresh_quota'];delete ko['startup.refresh_quota'];
assert.equal(m('startup.refresh_quota',{value1:'<img onerror=x>'}),'Refresh <img onerror=x> quota');ko['startup.refresh_quota']=original;
const title=()=>m('room.phase',{value1:2,value2:4,value3:'Literal {value1} <b>x</b>'});
i18n.bind(input,title,'title');
assert.equal(input.getAttribute('title'),'2/4단계 · Literal {value1} <b>x</b>');
i18n.set('en');assert.equal(dynamic.textContent,'Refresh my agent {value2} quota');
assert.equal(input.getAttribute('title'),'Phase 2/4 · Literal {value1} <b>x</b>');
i18n.text(fixed,'Workspace settings');i18n.set('ko');assert.equal(fixed.textContent,'Workspace settings');
i18n.set('xx');assert.equal(i18n.current(),'en');assert.equal(switches,4);
const mixed=new Node();const icon=new Node();icon.textContent='✓';i18n.label(mixed,()=>m('startup.drag_to_reorder'));mixed.append(icon);body.append(mixed);
const evidence=new Node('PRE');body.append(evidence);i18n.bind(evidence,()=>m('common.no_output'));const line=new Node();line.textContent='Workspace settings + literal diff';i18n.replace(evidence,line);
i18n.set('ko');assert.equal(mixed.textContent,'드래그해서 순서 변경✓');assert.equal(evidence.textContent,'Workspace settings + literal diff');
i18n.set('en');assert.equal(mixed.textContent,'Drag to reorder✓');assert.equal(evidence.textContent,'Workspace settings + literal diff');
console.log('ID lookup, fallback, interpolation, live bindings and user-content preservation passed');
