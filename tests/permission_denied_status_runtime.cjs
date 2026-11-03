// GitHub #341: a task the CLI refused is shown as its own state ("Permission denied"), red like a
// failure, in both languages.
const fs = require('node:fs'), vm = require('node:vm'), assert = require('node:assert/strict');
const lines = fs.readFileSync('src/ui/room.js', 'utf8').split('\n');
const pick = start => lines.find(l => l.startsWith(start));
const code = ['function statusLight(', 'const pretty =', 'const STATUS_TONE =', 'const statusTone =', 'const STATUS_LABELS =', 'const statusLabel ='].map(pick);
assert.ok(code.every(Boolean), 'room.js defines the status helpers');
const catalogs = { en: require('../src/ui/messages/en.json'), ko: require('../src/ui/messages/ko.json') };
for (const [language, messages] of Object.entries(catalogs)) {
  const context = vm.createContext({ AC_MESSAGES: catalogs, acMessage: id => messages[id] ?? id });
  vm.runInContext(code.join('\n') + '\nthis.out = [statusLight, statusTone, statusLabel];', context);
  const [light, tone, label] = context.out;
  assert.equal(light('permission_denied'), 'red');
  assert.equal(tone('permission_denied'), 'bad');
  assert.equal(label('permission_denied'), messages['room.permission_denied'], language);
  assert.notEqual(label('permission_denied'), label('failed'), `${language}: not shown as a plain failure`);
}
