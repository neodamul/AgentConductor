// GitHub #398 / R-210: a centered result popup shows the whole outcome of a completed goal at once.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const html=fs.readFileSync('src/ui/room.html','utf8'),room=fs.readFileSync('src/ui/room.js','utf8'),source=fs.readFileSync('src/ui/room_result.js','utf8'),ui=fs.readFileSync('src/ui.rs','utf8');
const dialog=html.match(/<dialog id="result-dialog"[\s\S]*?<\/dialog>/)[0];
for(const id of ['result-title','result-target','result-metrics','result-files','result-tests','result-lessons','result-close'])assert.match(dialog,new RegExp(`id="${id}"`),id);
assert.doesNotMatch(dialog,/<details|new.goal|start.new/i,'everything is shown at once, no expander and no new-goal shortcut');
assert.match(html,/id="show-result"[^>]*hidden/,'the rail has a Result button, hidden until the run is completed');
assert.match(room,/watchResult\(snapshot\)/);assert.match(ui,/room_result\.js/);

const context=vm.createContext({});vm.runInContext(source,context);
const run=code=>JSON.parse(JSON.stringify(vm.runInContext(code,context)));
// Changed files come from the candidate diff with +/- counts.
const diff=['diff --git a/src/a.rs b/src/a.rs','index 1..2 100644','--- a/src/a.rs','+++ b/src/a.rs','@@ -1 +1,2 @@','-old','+new','+more','diff --git a/docs/b.md b/docs/b.md','--- a/docs/b.md','+++ b/docs/b.md','@@ -0,0 +1 @@','+text'].join('\n');
assert.deepEqual(run(`resultFiles(${JSON.stringify(diff)})`),[{path:'src/a.rs',add:2,del:1},{path:'docs/b.md',add:1,del:0}]);
assert.deepEqual(run("resultFiles('')"),[]);assert.deepEqual(run('resultFiles(null)'),[]);
// Passed tests are the ones that exited 0 without timing out.
assert.deepEqual(run("resultTests([{task:'t1',exit_code:0},{task:'t2',exit_code:1},{task:'t3',exit_code:0,timed_out:true}])"),{passed:1,total:3});
assert.deepEqual(run('resultTests([])'),{passed:0,total:0});
// The popup opens once when a run completes while watched, never for a run that was already finished.
assert.equal(run("resultTransition(null,'completed')"),false,'first snapshot of a finished run: no popup');
assert.equal(run("resultTransition('running','completed')"),true);
assert.equal(run("resultTransition('approved','completed')"),true);
assert.equal(run("resultTransition('completed','completed')"),false);
assert.equal(run("resultTransition('running','failed')"),false);
// Lessons are gathered from the step lessons and the accepted knowledge of every task.
assert.deepEqual(run("resultLessons([{step_lessons:[{task:'t1',response:'Scope the test.'}],knowledge:[{task:'t1',lesson:{response:'Keep it small.'}}]},{step_lessons:[],knowledge:[]},{}])"),['Scope the test.','Keep it small.']);
assert.deepEqual(run("resultLessons([{step_lessons:[{response:'same'}],knowledge:[{lesson:{response:'same'}}]}])"),['same'],'duplicates are listed once');
console.log('Result popup: markup, files, tests, transition and lessons passed');
