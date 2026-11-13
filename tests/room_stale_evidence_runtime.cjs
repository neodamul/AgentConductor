// GitHub #794: a newly opened Room clears evidence surfaces before requesting its snapshot.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const source = (fs.readFileSync('src/ui/room.js', 'utf8') + fs.readFileSync('src/ui/room_render.js', 'utf8'));
assert.match(source, /function resetReviewEvidence\(\)[\s\S]*?\$\('review-summary'\)\.replaceChildren\(\)[\s\S]*?\$\('test-results'\)\.replaceChildren\(\)[\s\S]*?\$\('approve'\)\.disabled=true;\$\('merge'\)\.disabled=true;/, 'prior review and approval surfaces are cleared');
assert.match(source, /resetReviewEvidence\(\);\s*let snapshot/, 'clear evidence synchronously before the new Room snapshot is loaded');
// Exercise the guards instead of matching one exact spelling of the async function.
const vm=require('node:vm');
const loader=source.slice(source.indexOf('async function loadReview('),source.indexOf('// GitHub #684:',source.indexOf('async function loadReview(')));
async function rejectsLate(change){
 let release;const context={snapshot:{run_dir:'run-a'},roomStopped:false,review:null,request:()=>new Promise(resolve=>release=resolve),notice:message=>{throw Error(message);}};
 vm.createContext(context);vm.runInContext(loader,context);const pending=context.loadReview();change(context);release({report:{},test_output:[],diff:''});await pending;
 assert.equal(context.review,null,'obsolete evidence was not adopted');
}
(async()=>{
 await rejectsLate(context=>{context.roomStopped=true;});
 await rejectsLate(context=>{context.snapshot={run_dir:'run-b'};});
 console.log('room evidence reset: initial surfaces cleared; stopped and changed-run responses ignored');
})().catch(error=>{console.error(error);process.exitCode=1;});
