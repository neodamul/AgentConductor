// A hover measurement must describe the same hovered element it validates.
const assert = require('node:assert/strict'), fs = require('node:fs'), vm = require('node:vm');
const source = fs.readFileSync('tests/design_parity/parity.cjs', 'utf8');
const start = source.indexOf('const MEASURE = '), end = source.indexOf('\nasync function visit', start);
const measure = vm.runInNewContext(source.slice(start, end) + '; MEASURE');
let hovered = false;
const element = { getBoundingClientRect: () => ({left: 0, top: 0, width: 20, height: 22}), matches: () => hovered };
const context = {document: {querySelectorAll: () => [element]}, getComputedStyle: () => ({backgroundColor: hovered ? 'red' : 'gray'})};
assert.equal(vm.runInNewContext(measure('button', 0, ['backgroundColor'], null, null, true), context), null);
hovered = true;
assert.equal(vm.runInNewContext(measure('button', 0, ['backgroundColor'], null, null, true), context).backgroundColor, 'red');
hovered = false;
assert.equal(vm.runInNewContext(measure('button', 0, ['backgroundColor']), context).backgroundColor, 'gray');
console.log('hover state and computed style are measured atomically');
