// GitHub #414: compare the app UI with the approved design mock (docs/design/approved-goal-ui/preview.html)
// at the same viewport, element by element: geometry and computed style within a pixel tolerance, plus the
// share of differing pixels in the element's screenshot crop. Usage:
//   node tests/design_parity/parity.cjs [--out dir] [--only name] | --probe mock|app <selector> [url-suffix]
// (--probe app honours PROBE_SETUP / PROBE_RUN: simulator state to set, and a script to run, before measuring)
// Exit 0 = all within tolerance, 1 = a difference, 77 = skipped (no Chrome). Spec: spec.cjs.
const fs = require('fs'), path = require('path'), { execFileSync } = require('child_process');
const { findChrome, launch } = require('./cdp.cjs'), { decode, diffRatio } = require('./png.cjs');
const { pathToFileURL } = require('url'), repo = path.resolve(__dirname, '../..'), MOCK = pathToFileURL(path.join(repo, 'docs/design/approved-goal-ui/preview.html')).href;
const THEME = process.env.PARITY_THEME === 'dark' ? 'dark' : 'light';
const argv = process.argv.slice(2), flag = n => { const i = argv.indexOf(n); return i < 0 ? null : argv[i + 1]; };
const chrome = findChrome();
if (!chrome) { console.log('design parity skipped: no Chrome found (set CHROME)'); process.exit(77); }

const out = flag('--out') || fs.mkdtempSync(path.join(require('os').tmpdir(), 'ac-parity-out-'));
fs.mkdirSync(out, { recursive: true });
execFileSync('node', [path.join(repo, 'tests/ui_sim/build.cjs'), repo, path.join(out, 'app.html')], { stdio: 'ignore' });
const APP = pathToFileURL(path.join(out, 'app.html')).href;
const icon = f => 'data:image/svg+xml;base64,' + fs.readFileSync(path.join(repo, 'third_party/lobe-icons', f)).toString('base64');
const ICON_MAP = `{claude:${JSON.stringify(icon('claude-code-color.svg'))},codex:${JSON.stringify(icon('codex-color.svg'))}}`;
const PRESET = `SIM.home.icons=${ICON_MAP};SIM.room.icons=${ICON_MAP};`;

// Nth visible match; `mark` is the logo inside the element (mock) or the content box of an <img> (app).
const MEASURE = (sel, index, props, mark) => `(() => { const e = [...document.querySelectorAll(${JSON.stringify(sel)})].filter(x => x.getBoundingClientRect().width > 0)[${index}]; if (!e) return null;
  const r = e.getBoundingClientRect(), c = getComputedStyle(e), o = { x: r.left, y: r.top, width: r.width, height: r.height };
  for (const p of ${JSON.stringify(props)}) o[p] = c[p];
  const k = ${JSON.stringify(mark || null)} && e.querySelector(${JSON.stringify(mark || 'x')});
  if (${JSON.stringify(mark || null)} === 'content') o.mark = r.width - 2 * (parseFloat(c.borderLeftWidth) + parseFloat(c.paddingLeft)); else if (k) o.mark = k.getBoundingClientRect().width;
  return o; })()`;

async function visit(browser, side, spec) {
  if (side === 'mock') { await browser.goto('about:blank'); await browser.goto(MOCK + (spec.url || '?theme=light#0').replace('theme=light', 'theme=' + THEME)); } else {
    await browser.goto(APP); await browser.eval(`sessionStorage.setItem('SIM_PRESET', ${JSON.stringify(PRESET + (spec.setup || ''))}); location.reload()`);
    await new Promise(r => setTimeout(r, 1200));
  }
  if (side === 'app') await browser.eval(`document.documentElement.dataset.theme = ${JSON.stringify(THEME)}`);
  if (spec.run) await browser.eval(spec.run);
  await new Promise(r => setTimeout(r, 600));
  // Pulses and slides are checked by their own tests; freeze them so the pixels are comparable.
  await browser.eval("document.head.append(Object.assign(document.createElement('style'), { textContent: '*,*::before,*::after{animation:none!important;transition:none!important}' }))");
  await new Promise(r => setTimeout(r, 150));
}

(async () => {
  const browser = await launch(chrome);
  try {
    if (argv[0] === '--probe') {
      await visit(browser, argv[1], { url: argv[3], setup: process.env.PROBE_SETUP, run: process.env.PROBE_RUN });
      console.log(JSON.stringify(await browser.eval(`[...document.querySelectorAll(${JSON.stringify(argv[2])})].slice(0, 6).map(e => { const r = e.getBoundingClientRect(), c = getComputedStyle(e); return { cls: String(e.className.baseVal ?? e.className), w: r.width, h: r.height, x: r.left, y: r.top, bw: c.borderTopWidth, bc: c.borderTopColor, pad: c.paddingTop, rad: c.borderTopLeftRadius }; })`), null, 1));
      return;
    }
    const failures = [], specs = require('./spec.cjs').filter(x => !flag('--only') || x.name === flag('--only')), ran = specs.length;
    // Measure grouped by page state so each state is loaded once (the run is dominated by page loads).
    const key = (side, x) => side === 'mock' ? 'mock|' + (x.mock.url || '?theme=light#0') : 'app|' + (x.app.setup || '') + '|' + (x.app.run || '');
    const jobs = []; for (const x of specs) if (!x.assert) for (const side of ['mock', 'app']) jobs.push({ x, side });
    jobs.sort((p, q) => (key(p.side, p.x) < key(q.side, q.x) ? -1 : key(p.side, p.x) > key(q.side, q.x) ? 1 : 0));
    const got = {}; let current = null;
    for (const { x, side } of jobs) {
      const k = key(side, x); if (k !== current) { await visit(browser, side, x[side]); current = k; }
      const s = x[side];
      if (x.hover) { const c = await browser.eval(MEASURE(s.selector, s.index || 0, [], null)); if (c) await browser.hover(c.x + c.width / 2, c.y + c.height / 2); }
      if (x.focus) { await browser.tab(); await browser.eval(`[...document.querySelectorAll(${JSON.stringify(s.selector)})].filter(e => e.getBoundingClientRect().width > 0)[${s.index || 0}]?.focus()`); }
      const m = await browser.eval(MEASURE(s.selector, s.index || 0, x.props || [], s.mark));
      if (x.focus) await browser.eval('document.activeElement && document.activeElement.blur()');
      if (x.hover) await browser.hover(0, 0);
      got[x.name] = got[x.name] || { shots: {} }; got[x.name][side] = m;
      if (!m) { failures.push(`${x.name}: ${side} element not found (${s.selector})`); continue; }
      got[x.name].shots[side] = await browser.shot({ x: m.x, y: m.y, width: m.width, height: m.height });
      fs.writeFileSync(path.join(out, `${x.name}.${side}.png`), got[x.name].shots[side]);
    }
    for (const spec of specs) {
      if (spec.assert) { // behaviour check on the app only: `assert` is evaluated after `run` and must be true
        if (spec.viewport) await browser.viewport(spec.viewport[0], spec.viewport[1]);
        await visit(browser, 'app', spec.app); const ok = await browser.eval(spec.assert);
        if (spec.viewport) await browser.viewport(1280, 960);
        if (ok !== true) failures.push(`${spec.name}: assertion failed (${spec.assert})`); continue;
      }
      const measured = got[spec.name], shots = measured.shots;
      if (!measured.mock || !measured.app) continue;
      const tol = 0.01, cmp = (label, a, b) => { for (const k of Object.keys(a)) { if (k === 'shots' || (spec.keys && !spec.keys.includes(k))) continue;
        const x = a[k], y = b[k], num = typeof x === 'number'; if (num ? Math.abs(x - y) > (k === 'x' || k === 'y' ? Infinity : tol) : String(x) !== String(y)) failures.push(`${spec.name}${label}.${k}: mock ${x} app ${y}`); } };
      cmp('', measured.mock, measured.app);
      if (shots.mock && shots.app && spec.pixel !== false) { const d = diffRatio(decode(shots.mock), decode(shots.app)); console.log(`  ${spec.name}: ${(d.ratio * 100).toFixed(2)}% of pixels differ`); if (d.ratio > 0) failures.push(`${spec.name}: ${(d.ratio * 100).toFixed(1)}% of pixels differ${d.sizeMismatch ? ' (size ' + d.sizeMismatch.join(',') + ')' : ''}`); }
    }
    console.log(`design parity: ${ran} element(s), ${failures.length} difference(s); captures in ${out}`);
    failures.forEach(f => console.log('  - ' + f));
    process.exitCode = failures.length ? 1 : 0;
  } finally { await browser.close(); }
})().catch(e => { console.error(e); process.exit(2); });
