const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync('src/ui/room.js','utf8');const buttons=new Map([['approve',{}]]),calls=[];
const review={revision:'exact',truncated:true,report:{candidate:{commit:'verified-candidate'},diff_hash:'complete-patch-hash',tests:[{}]}};
const context=vm.createContext({review,snapshot:{revision:'exact'},$:id=>buttons.get(id),acMessage:id=>id,decision:async()=>true,act:async(...args)=>calls.push(args)});
const gate=source.slice(source.indexOf('const currentReview='),source.indexOf(';',source.indexOf('const currentReview=')));
assert.equal(vm.runInContext(gate+';currentReview',context),true,'preview truncation must not disable a current review');
const start=source.indexOf("$('approve').onclick="),end=source.indexOf("\n",start);vm.runInContext(source.slice(start,end),context);
(async()=>{await buttons.get('approve').onclick();assert.equal(calls.length,1,'a shortened preview can approve the verified full candidate');assert.equal(calls[0][0].sha,'verified-candidate');assert.equal(calls[0][1],'exact');calls.length=0;context.snapshot.revision='stale';await buttons.get('approve').onclick();assert.equal(calls.length,0,'stale review cannot approve');console.log('Diff preview approval: truncation allowed; exact candidate/revision preserved');})().catch(e=>{console.error(e);process.exitCode=1;});
