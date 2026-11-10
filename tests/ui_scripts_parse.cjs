// Every UI script must parse: the page loads them as one document, where a single syntax error stops all of them.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const dir = 'src/ui/';
for (const name of fs.readdirSync(dir).filter(f => f.endsWith('.js'))) {
  const source = fs.readFileSync(dir + name, 'utf8');
  // room.js, room_activity.js, room_result.js and room_rail.js are embedded inside window.startRoom=function(){ ... }.
  const wrapped = ['room.js', 'room_activity.js', 'room_result.js', 'room_rail.js'].includes(name) ? 'function startRoom(){' + source + '\n}' : source;
  assert.doesNotThrow(() => new vm.Script(wrapped, { filename: name }), name + ' parses');
}
console.log('UI scripts parse');
