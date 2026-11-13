// GitHub #934: Claude usage that could not be read for lack of a project folder is queried again as soon as a folder is set.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const source=fs.readFileSync('src/ui/startup.js','utf8');
const start=source.indexOf('function autoDiscover(){');assert(start>=0,'autoDiscover exists');
const body=source.slice(start,source.indexOf('\n',start));
const calls=[];
const ctx={state:null,quotaRefreshed:new Map(),refreshQuota:id=>{calls.push(id);ctx.quotaRefreshed.set(id,Date.now());},busy:()=>false,dirty:false,planDirty:false,draft:{profiles:[]},addedAgents:new Set(),action:()=>{}};
vm.createContext(ctx);vm.runInContext(body+';this.autoDiscover=autoDiscover;',ctx);
const snapshot=repository=>({config:{repository,profiles:[{id:'claude',parser:'claude'}]},quota_refresh_due:{},quota_indicators:{claude:{error_code:'no_project_folder',error:'x'}}});
ctx.state=snapshot('');ctx.autoDiscover();
assert.deepEqual(calls,[],'no folder: no query');
ctx.state=snapshot('/work/project');ctx.autoDiscover();
assert.deepEqual(calls,['claude'],'folder entered: usage is queried without a retry click');
ctx.autoDiscover();
assert.deepEqual(calls,['claude'],'a repeated folder error does not loop');
console.log('quota_folder_934 ok');
