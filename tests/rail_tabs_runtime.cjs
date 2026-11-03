// GitHub #392 / R-210: the left rail has Agents and Phases & tasks tabs on both screens.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const html=fs.readFileSync('src/ui/startup.html','utf8'),room=fs.readFileSync('src/ui/room.html','utf8');
const nav=html.match(/<nav id="rail-tabs"[\s\S]*?<\/nav>/)[0];
assert.deepEqual([...nav.matchAll(/id="rail-tab-([a-z]+)"/g)].map(m=>m[1]),['agents','tasks']);
// Every agent control stays in the agents pane, which is the node that moves between the screens.
const pane=html.match(/<div id="rail-pane-agents"[\s\S]*?<div id="start-profiles"><\/div><\/div>/)[0];
for(const id of ['start-add','start-profiles'])assert.ok(pane.includes(`id="${id}"`),id+' stays in the moving agents pane');
assert.match(room,/<aside class="rail"><div id="rail-room"/);
for(const id of ['sessions','open-review','open-tools','run-state'])assert.ok(room.includes(`id="${id}"`),id+' stays in the Room rail');

function node(id){return{id,hidden:false,tabIndex:0,attrs:{},children:[],setAttribute(k,v){this.attrs[k]=v;},focus(){focused=this;},replaceChildren(...c){this.children=c;}};}
let focused,onRun=false;
const ids=['rail-tab-agents','rail-tab-tasks','rail-pane-agents','rail-outline','rail-outline-body','rail-outline-empty','rail-room'];
const nodes=Object.fromEntries(ids.map(i=>[i,node(i)]));
const document={getElementById:i=>nodes[i],body:{classList:{contains:()=>onRun}},createElement:tag=>({tag,children:[],className:'',append(...c){this.children.push(...c);}}),createComment:()=>({})};
const window={};vm.runInNewContext(fs.readFileSync('src/ui/rail.js','utf8'),{window,document});
const rail=window.acRail,shown=()=>({agents:!nodes['rail-pane-agents'].hidden,outline:!nodes['rail-outline'].hidden,room:!nodes['rail-room'].hidden,tab:nodes['rail-tab-agents'].attrs['aria-selected']==='true'?'agents':'tasks'});

// Steps 1-2 open on Agents; 3-6 on Phases & tasks.
rail.forStep('workspace');assert.deepEqual(shown(),{agents:true,outline:false,room:false,tab:'agents'});
rail.forStep('team');assert.equal(shown().tab,'agents');
rail.forStep('review');assert.deepEqual(shown(),{agents:false,outline:true,room:false,tab:'tasks'},'on the Workspace screen the tasks tab shows the plan outline');
onRun=true;rail.forStep('tasks');assert.deepEqual(shown(),{agents:false,outline:false,room:true,tab:'tasks'},'in the Room it shows the session rail');
rail.select('agents');assert.deepEqual(shown(),{agents:true,outline:false,room:false,tab:'agents'},'the agent panel is the same node, visible in the Room too');
rail.select('nope');assert.equal(shown().tab,'agents');
assert.equal(nodes['rail-tab-agents'].tabIndex,0);assert.equal(nodes['rail-tab-tasks'].tabIndex,-1);

// The outline lists the plan's tasks, or says there is no plan yet.
rail.outline(null);assert.equal(nodes['rail-outline-empty'].hidden,false);assert.equal(nodes['rail-outline-body'].children.length,0);
rail.outline({tasks:[{id:'task-1',prompt:'Write the parser\nmore detail'},{id:'task-2',prompt:'x'.repeat(300)}]});
assert.equal(nodes['rail-outline-empty'].hidden,true);const list=nodes['rail-outline-body'].children[0];assert.equal(list.children.length,2);
assert.deepEqual(list.children[0].children.map(c=>c.textContent??c),['task-1',' ','Write the parser']);
assert.equal(list.children[1].children[2].textContent.length,120);

// Keyboard: arrows and Home/End move between the two tabs and select.
rail.bind();let prevented=0;const press=(name,key)=>nodes['rail-tab-'+name].onkeydown({key,preventDefault(){prevented++;}});
rail.select('agents');press('agents','ArrowRight');assert.equal(shown().tab,'tasks');assert.equal(focused,nodes['rail-tab-tasks']);
press('tasks','ArrowLeft');assert.equal(shown().tab,'agents');press('agents','End');assert.equal(shown().tab,'tasks');press('tasks','Home');assert.equal(shown().tab,'agents');
press('agents','x');assert.equal(prevented,4,'other keys are left alone');
nodes['rail-tab-tasks'].onclick();assert.equal(shown().tab,'tasks');
console.log('Rail tabs: markup, step defaults, outline, keyboard passed');
