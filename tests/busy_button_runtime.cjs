// User request 2026-10-14: a pressed button shows its work is running, distinct from disabled.
const vm = require('node:vm'), fs = require('node:fs'), assert = require('node:assert/strict');
const source = fs.readFileSync('src/ui/i18n.js', 'utf8');
const press = source.slice(source.indexOf('const AC_PRESS='));
let clickHandler = null;
const document = { addEventListener(kind, handler) { if (kind === 'click') clickHandler = handler; } };
function button(visible = true) {
  const classes = new Set(), attrs = {};
  return {
    dataset: {}, closest() { return this; }, getClientRects: () => (visible ? [{}] : []),
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

  // GitHub #856: Next clicks the hidden prepare button; the bar stays on Next.
  const next = button(), prepare = button(false);
  clickHandler({ target: next }); clickHandler({ target: prepare });
  let done;
  const plan = AC_PRESS.track(new Promise(resolve => { done = resolve; }));
  assert.ok(next.classList.contains('is-busy') && !prepare.classList.contains('is-busy'), 'the visible pressed button carries the bar');
  AC_PRESS.hold(next, false);
  assert.ok(next.classList.contains('is-busy'), 'releasing a hold never clears a running request mark');
  done(); await plan;
  assert.ok(!next.classList.contains('is-busy'), 'cleared when that request ends');
  const regenerate = button(); clickHandler({ target: regenerate });
  let slow; const first = AC_PRESS.track(new Promise(resolve => { slow = resolve; }));
  await AC_PRESS.track(Promise.resolve());
  assert.ok(regenerate.classList.contains('is-busy'), 'a quick request does not end the bar while another from the same press runs');
  slow(); await first;
  assert.ok(!regenerate.classList.contains('is-busy'), 'the bar ends with the last request');
  console.log('Busy buttons: request mark, background hold, release and hidden-proxy press passed');
})().catch(error => { console.error(error); process.exit(1); });
