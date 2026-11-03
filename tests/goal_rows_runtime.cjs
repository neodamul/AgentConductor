// GitHub #402 / DD-222: Goal-list row actions. The confirmed deletion retries the same reviewed Goal and folder with a
// fresh revision while owned work drains, and never reports success unless the backend says deleted.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const html=fs.readFileSync('src/ui/startup.html','utf8'),source=fs.readFileSync('src/ui/goal_rows.js','utf8'),startup=fs.readFileSync('src/ui/startup.js','utf8'),document_rs=fs.readFileSync('src/ui/setup/document.rs','utf8');
const dialog=html.match(/<dialog id="start-delete-goal"[\s\S]*?<\/dialog>/)[0];
for(const id of ['start-delete-goal-name','start-delete-goal-path','start-delete-goal-status','start-delete-goal-confirm'])assert.match(dialog,new RegExp(`id="${id}"`),id);
assert.match(dialog,/<button[^>]*value="cancel"[^>]*autofocus/,'Cancel is the focused default');
assert.doesNotMatch(dialog,/id="start-delete-goal-confirm"[^>]*autofocus/,'the destructive button is never the default');
assert.match(dialog,/permanently|data-i18n="startup\.delete_goal_nobackup"/,'the no-backup warning is shown');
assert.match(document_rs,/goal_rows\.js/);assert.match(startup,/acGoalRows\?\.decorate/);
const context=vm.createContext({window:{},setTimeout});vm.runInContext(source,context);
const rows=context.window.acGoalRows,plain=v=>JSON.parse(JSON.stringify(v));
const target={slug:'a',goal:'Goal A',root:'/w/a'};
(async()=>{
  const calls=[];let revision=0;const fresh=()=>`r${++revision}`;
  // Deleted at once.
  let r=await rows.confirmDelete(async p=>{calls.push(p);return {deleted:true,stopping:false,slug:'a',changed:true};},fresh,target,{sleep:async()=>{}});
  assert.deepEqual(plain(r),{deleted:true,changed:true});assert.deepEqual(plain(calls[0]),{kind:'delete_goal',revision:'r1',slug:'a',goal:'Goal A',root:'/w/a'});
  // Pending twice: the same confirmed Goal/folder is retried with a fresh revision each time.
  calls.length=0;let n=0;const pending=[];
  r=await rows.confirmDelete(async p=>{calls.push(p);return ++n<3?{deleted:false,stopping:true}:{deleted:true,stopping:false,slug:'a',changed:false};},fresh,target,{sleep:async()=>{},onPending:i=>pending.push(i)});
  assert.equal(r.deleted,true);assert.equal(calls.length,3);assert.deepEqual(plain(calls.map(c=>c.revision)),['r2','r3','r4'],'a fresh revision per attempt');
  assert.ok(calls.every(c=>c.slug==='a'&&c.goal==='Goal A'&&c.root==='/w/a'),'never retargets');assert.deepEqual(plain(pending),[0,1]);
  // A backend error is surfaced, not swallowed or turned into success.
  await assert.rejects(rows.confirmDelete(async()=>{throw new Error('Goal folder changed');},fresh,target,{sleep:async()=>{}}),/Goal folder changed/);
  // Never optimistic: an endlessly pending deletion gives up without claiming success.
  r=await rows.confirmDelete(async()=>({deleted:false,stopping:true}),fresh,target,{sleep:async()=>{},tries:3});
  assert.deepEqual(plain(r),{deleted:false,timedOut:true});
  console.log('Goal rows: confirmation markup, fresh-revision retry, error and timeout passed');
})().catch(e=>{console.error(e);process.exit(1)});
