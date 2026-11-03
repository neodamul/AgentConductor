// GitHub #396 / R-210: the Goal list is a right slide-over opened from a counted button on both screens.
const assert=require('node:assert/strict'),fs=require('node:fs');
const html=fs.readFileSync('src/ui/startup.html','utf8'),css=fs.readFileSync('src/ui/startup.css','utf8'),js=fs.readFileSync('src/ui/startup.js','utf8'),room=fs.readFileSync('src/ui/room.html','utf8');
const dialog=html.match(/<dialog id="start-goals"[\s\S]*?<\/dialog>/)[0];
assert.match(dialog,/class="slide-panel"/);assert.match(dialog,/aria-labelledby="start-goals-title"/);
assert.match(dialog,/id="start-goals-title"/,'the panel has a visible title');
assert.match(dialog,/id="start-goals-close"[^>]*type="button"|type="button"[^>]*id="start-goals-close"/,'a close control');
assert.match(dialog,/id="start-goal-list"/,'the existing list container is kept');
// The opener is an outlined button with an icon and a count badge, in the Workspace heading.
const opener=html.match(/<button[^>]*id="start-goal-list-open"[\s\S]*?<\/button>/)[0];
assert.match(opener,/<svg/);assert.match(opener,/id="start-goal-count"/);assert.match(opener,/aria-haspopup="dialog"/);
// The same button moves into the Room's goal bar (room.html stays usable on its own).
assert.ok(!room.includes('start-goals'));
assert.match(js,/function placeGoalList\(onRun\)/);assert.match(js,/placeGoalList\(true\)/);assert.match(js,/placeGoalList\(false\)/);
assert.match(js,/q\('goals'\)\.showModal\(\)/);assert.match(js,/q\('goals-close'\)\.onclick/);
// Slide-over styling: pinned right, full height, moves in only when motion is allowed.
assert.match(css,/dialog\.slide-panel\{[^}]*margin:0 0 0 auto/);assert.match(css,/dialog\.slide-panel[^}]*height:100(?:%|vh)/);
assert.match(css,/prefers-reduced-motion[^}]*\{[^}]*slide-panel/);
console.log('Goal panel: markup, opener, Room button and slide-over styling present');
