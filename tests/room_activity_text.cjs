// GitHub #100: the task header's activity line states facts and elapsed time, never a cause.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
const grab = name => { const s = source.indexOf(`function ${name}(`); assert.ok(s >= 0, `room.js defines ${name}`); return source.slice(s, source.indexOf('\n', s)); };
const templates = {
  'room.activity_last_output': 'last {value1}', 'room.activity_no_output': 'no output',
  'room.activity_tool': 'tool {value1} {value2}', 'room.activity_process_present': 'alive',
  'room.activity_process_absent': 'absent', 'room.activity_process_unknown': 'unverifiable',
};
const context = vm.createContext({ acMessage: (id, v = {}) => (templates[id] || id).replace('{value1}', v.value1).replace('{value2}', v.value2) });
vm.runInContext(grab('formatAge') + grab('activityLines'), context);
const run = (a, extra = 0) => JSON.parse(vm.runInContext(`JSON.stringify(activityLines(${JSON.stringify(a)}, ${extra}))`, context));
assert.equal(vm.runInContext('formatAge(45000)', context), '45s');
assert.equal(vm.runInContext('formatAge(185000)', context), '3m 05s');
assert.equal(vm.runInContext('formatAge(3720000)', context), '1h 02m');
assert.deepEqual(run({ quiet_ms: 180000, tool: { name: 'Bash', running_ms: 240000 }, process: 'present' }, 5000),
  ['tool Bash 4m 05s', 'last 3m 05s', 'alive']);
assert.deepEqual(run({ quiet_ms: null, tool: null, process: 'absent' }), ['no output', 'absent']);
assert.deepEqual(run({ quiet_ms: 1000, tool: null, process: 'unknown' }), ['last 1s', 'unverifiable']);
assert.deepEqual(run({ quiet_ms: 1000, tool: null, process: 'not_started' }), ['last 1s']);
assert.deepEqual(run(null), []);
console.log('activity text ok');
const css = fs.readFileSync('src/ui/room.css', 'utf8');
assert.ok(source.includes("(session.status==='running'?' is-running':'')"), 'running rows are classed');
assert.ok(css.includes('.session.is-running .session-choice>:first-child{animation:attn-blink'), 'the running icon pulses');
assert.ok(/prefers-reduced-motion:reduce\)\{\.session\.is-running[^}]*animation:none/.test(css), 'reduced motion stops the pulse');
console.log('running icon ok');
// GitHub #117: the activity line keeps one fixed slot so the tabs below never move.
const html = fs.readFileSync('src/ui/room.html', 'utf8');
const rule = css.match(/\.task-activity\{[^}]*\}/)?.[0] || '';
for (const part of ['white-space:nowrap', 'overflow:hidden', 'text-overflow:ellipsis', 'min-height:', 'font-variant-numeric:tabular-nums']) assert.ok(rule.includes(part), `.task-activity has ${part}`);
assert.ok(!/id="task-activity"[^>]*\shidden/.test(html), 'the slot is never display:none');
const render = source.slice(source.indexOf('function renderActivity('), source.indexOf('\n', source.indexOf('function renderActivity(')));
assert.ok(!render.includes('.hidden'), 'toggling hidden shifts the layout');
assert.ok(render.includes("style.visibility"), 'the slot is hidden with visibility only');
console.log('activity slot ok');
