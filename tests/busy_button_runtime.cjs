// User request 2026-10-14: a pressed button shows its work is running, distinct from disabled.
const vm = require('node:vm'), fs = require('node:fs'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/i18n.js', 'utf8');
const press = source.slice(source.indexOf('const AC_PRESS='));
let clickHandler = null;
const document = { addEventListener(kind, handler) { if (kind === 'click') clickHandler = handler; } };
function button() {
  const classes = new Set(), attrs = {};
  return {
    dataset: {}, closest() { return this; },
    classList: { toggle(c, on) { on ? classes.add(c) : classes.delete(c); }, contains: c => classes.has(c) },
    setAttribute(k, v) { attrs[k] = v; }, removeAttribute(k) { delete attrs[k]; }, attrs,
  };
}
const AC_PRESS = vm.runInNewContext(press + ';AC_PRESS', { document, Date, Promise });
(async () => {
  const save = button();
  clickHandler({ target: save });
  let finish;
  const pending = AC_PRESS.track(new Promise(resolve => { finish = resolve; }));
  assert.ok(save.classList.contains('is-busy'), 'marked while the request runs');
  assert.equal(save.attrs['aria-busy'], 'true');
  finish('done');
  assert.equal(await pending, 'done');
  assert.ok(!save.classList.contains('is-busy'), 'cleared when the request ends');

  const run = button();
  clickHandler({ target: run });
  AC_PRESS.hold(run, true);
  await AC_PRESS.track(Promise.resolve());
  assert.ok(run.classList.contains('is-busy'), 'held while its background job runs');
  AC_PRESS.hold(run, false);
  assert.ok(!run.classList.contains('is-busy'), 'released when the job ends');
  console.log('Busy buttons: request mark, background hold and release passed');
})().catch(error => { console.error(error); process.exit(1); });
