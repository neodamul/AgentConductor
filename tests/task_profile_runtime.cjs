// GitHub #210: the plan review and the roles table show what a task runs with and which efforts a model honors.
// Extracts profileText and modelHint from startup.js and runs them with stub messages.
const fs=require('fs'),vm=require('vm'),assert=require('assert');
const src=fs.readFileSync('src/ui/startup.js','utf8');
const pick=(start)=>{const i=src.indexOf(start);const j=src.indexOf('\n',i);return src.slice(i,j);};
const code=[pick('  const TASK_TYPES='),pick('  function profileText('),pick('  function modelHint(')].join('\n');
const msgs=JSON.parse(fs.readFileSync('src/ui/messages/en.json','utf8'));
const ctx={acMessage:(id,v={})=>(msgs[id]||id).replace(/\{(\w+)\}/g,(m,k)=>k in v?v[k]:m),draft:{profiles:[
 {id:'a',execution_options:[{name:'effort',values:['low','medium','high'],model_values:{big:['low','medium','high'],small:['low']}}],agent_settings:{capabilities:{models:[{id:'small',traits:{best_for:['extract','creative']}}]}}},
 {id:'b'}]}};
vm.createContext(ctx);vm.runInContext(code+';this.profileText=profileText;this.modelHint=modelHint;',ctx);
assert.equal(ctx.profileText({task_type:'code',effort:'high',warnings:[]}),'Code · effort high');
assert.equal(ctx.profileText({task_type:'code',effort:'low',warnings:[{code:'effort_unsupported',asked:'high',applied:'low'}]}),'Code · effort low · effort high is not honored by the model, using low');
assert.ok(ctx.profileText({task_type:'agentic',effort:'high',warnings:[{code:'model_not_suited',model:'small',task_type:'agentic'}]}).includes('small is not a good fit for: Long agent work'));
assert.equal(ctx.modelHint('a','small'),'Effort: low · Suits: Classify or extract, Creative');
assert.equal(ctx.modelHint('a','big'),'Effort: low, medium, high · Suits: –');
assert.equal(ctx.modelHint('b','x'),'');
assert.equal(ctx.modelHint('zzz','x'),'');
console.log('page helpers ok');
