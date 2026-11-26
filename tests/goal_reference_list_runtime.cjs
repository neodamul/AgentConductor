const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const context = vm.createContext({window:{}});
vm.runInContext(fs.readFileSync('src/ui/goal_rows.js', 'utf8'), context);
const {listPage} = context.window.acGoalRows;
const source = (kind, id) => ({kind, id});
const goal = (slug, ...references) => ({slug, goal:slug, references});
const entries = [
  goal('legacy'), goal('twenty', source('jira', 'EMM-20')), goal('three', source('jira', 'EMM-3')),
  goal('older', source('confluence', '9007199254740992')),
  goal('newer', source('confluence', '9007199254740993')),
  ...Array.from({length:8}, (_, i) => goal('goal' + i))
];
entries[1].created_ms = 1000;
entries[2].created_ms = 3000;
entries[3].created_ms = 2000;
const rows = (query, sort, page) => JSON.parse(JSON.stringify(listPage(entries, query, sort, page)));
assert.deepEqual(rows('', undefined, 1).rows.slice(0,4).map(r=>r.slug), ['three','older','twenty','legacy']);
assert.deepEqual(rows('', 'created-asc', 1).rows.slice(0,4).map(r=>r.slug), ['twenty','older','three','legacy']);
assert.equal(rows('JIRA: emm-3', 'created-desc', 1).rows[0].slug, 'three');
assert.equal(rows('9007199254740993', 'created-desc', 1).rows[0].slug, 'newer');
assert.equal(rows('legacy', 'created-desc', 1).rows[0].slug, 'legacy');
assert.deepEqual(rows('', 'jira-asc', 1).rows.slice(0, 3).map(r => r.slug), ['three','twenty','legacy']);
assert.deepEqual(rows('', 'confluence-desc', 1).rows.slice(0, 3).map(r => r.slug), ['newer','older','legacy']);
assert.equal(rows('', 'created-desc', 1).rows.length, 10);
assert.equal(rows('', 'created-desc', 2).rows.length, 3);
assert.equal(rows('legacy', 'created-desc', 2).page, 1);
assert.equal(rows('missing', 'created-desc', 1).total, 0);
console.log('Goal history reference search, numeric order, and paging passed');
