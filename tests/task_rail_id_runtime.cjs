// GitHub #777: task IDs stay available to assistive technology while compact ordinals leave room for descriptions.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const source = fs.readFileSync('src/ui/room.js', 'utf8');
assert.match(source, /node\('code','T'\+\(index\+1\),'task-number'\)/, 'execution rows show compact sequential task numbers');
assert.match(source, /AC_I18N\.bind\(button,\(\)=>\(session\.id\+' · '\+\(taskLine\(plan\.prompt\)\|\|session\.id\)\),"title"\)/, 'full task ID and description remain available on the row');
console.log('task rail identity: compact ordinals and full task tooltip');
