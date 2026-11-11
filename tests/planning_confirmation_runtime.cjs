const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const source=fs.readFileSync('src/ui/startup.js','utf8');
const context={notice(){},acMessage:id=>id,state:{revision:'r1'},workspaceEpoch:0,draft:{goal:'Goal A',coordinator:'custom-cli',profiles:[{id:'custom-cli',coordinator_model:'local-v1',catalog:{models:['local-v1']}}]},workspaceReviewKey:JSON.stringify};
vm.createContext(context);
vm.runInContext(source.slice(source.indexOf('function planningContextKey('),source.indexOf('// DD-46:')),context);
(async()=>{
 const selection=await context.confirmPlanning();assert(selection);assert(context.planningUnchanged(selection));
 context.draft.profiles[0].coordinator_model='other';assert(!context.planningUnchanged(selection));assert.equal(await context.confirmPlanning(),null);
 context.draft.profiles[0].coordinator_model='';assert.equal(await context.confirmPlanning(),null);
 delete context.draft.profiles[0].coordinator_model;assert(await context.confirmPlanning());
 context.draft.coordinator='missing';assert.equal(await context.confirmPlanning(),null);
 context.draft.coordinator='custom-cli';const workspace=await context.confirmPlanning();context.workspaceEpoch++;assert(!context.planningUnchanged(workspace));
 console.log('Planning selection validates model, coordinator and workspace context');
})().catch(error=>{console.error(error);process.exitCode=1;});
