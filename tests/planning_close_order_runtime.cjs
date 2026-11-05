// #484: a queued native close event must not cancel a newly opened confirmation.
const assert=require("node:assert/strict"),fs=require("node:fs"),vm=require("node:vm");
const source=fs.readFileSync("src/ui/startup.js","utf8"),closed=[],nodes=new Map();
function q(id){if(!nodes.has(id))nodes.set(id,{open:false,dataset:{},showModal(){this.open=true;},close(value){this.returnValue=value;this.open=false;closed.push(()=>this.onclose?.());}});return nodes.get(id);}
const context={q,notice(){},acMessage:()=>"evidence",AC_I18N:{bind(){}},workspaceEpoch:0,workspaceReviewKey:JSON.stringify,state:{revision:"r1"},draft:{goal:"Goal",coordinator:"claude",profiles:[{id:"claude"}]}};
vm.createContext(context);vm.runInContext(source.slice(source.indexOf("function planningContextKey("),source.indexOf("// DD-46:")),context);
const deadline=setTimeout(()=>{console.error("Planning decision did not settle");process.exit(1);},3000);
(async()=>{
 const dialog=q("planning-confirm"),first=context.confirmPlanning();
 dialog.onsubmit?.({submitter:{value:"cancel"}});dialog.close("cancel");
 const second=context.confirmPlanning();let settled=false;second.then(()=>settled=true);
 closed.shift()();await new Promise(setImmediate);
 assert.equal(settled,false,"the previous close event must not cancel the new review");
 dialog.onsubmit?.({submitter:{value:"confirm"}});dialog.close("confirm");
 assert.equal(await first,null,"the previous cancellation settles its own request");
 assert(await second,"the new explicit confirmation is accepted");
 closed.shift()();
 console.log("Queued close/reopen: cancellation and acceptance stay bound to their own invocation");
})().catch(error=>{console.error(error);process.exitCode=1;}).finally(()=>clearTimeout(deadline));
