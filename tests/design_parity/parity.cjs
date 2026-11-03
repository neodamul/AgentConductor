// GitHub #414: compare the app UI with the approved design mock (docs/design/approved-goal-ui/preview.html)
// at the same viewport, element by element: geometry and computed style within a pixel tolerance, plus the
// share of differing pixels in the element's screenshot crop. Usage:
//   node tests/design_parity/parity.cjs [--out dir] [--only name] | --probe mock|app <selector> [url-suffix]
// Exit 0 = all within tolerance, 1 = a difference, 77 = skipped (no Chrome). Spec: spec.cjs.
const fs = require('fs'), path = require('path'), { execFileSync } = require('child_process');
const { findChrome, launch } = require('./cdp.cjs'), { decode, diffRatio } = require('./png.cjs');
const { pathToFileURL } = require('url'), repo = path.resolve(__dirname, '../..'), MOCK = pathToFileURL(path.join(repo, 'docs/design/approved-goal-ui/preview.html')).href;
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
  if (side === 'mock') await browser.goto(MOCK + (spec.url || '?theme=light#0')); else {
    await browser.goto(APP); await browser.eval(`sessionStorage.setItem('SIM_PRESET', ${JSON.stringify(PRESET + (spec.setup || ''))}); location.reload()`);
    await new Promise(r => setTimeout(r, 1200));
  }
  if (spec.run) await browser.eval(spec.run);
  await new Promise(r => setTimeout(r, 600));
  // Pulses and slides are checked by their own tests; freeze them so the pixels are comparable.
  await browser.eval("document.head.append(Object.assign(document.createElement('style'), { textContent: '*,*::before,*::after{animation:none!important;transition:none!important}' }))");
  await new Promise(r => setTimeout(r, 150));
}

(async () => {
  let browser = await launch(chrome); let viewport = JSON.stringify({width:1280,height:960});
  try {
    if (argv[0] === '--probe') {
      await visit(browser, argv[1], { url: argv[3] });
      console.log(JSON.stringify(await browser.eval(`[...document.querySelectorAll(${JSON.stringify(argv[2])})].slice(0, 6).map(e => { const r = e.getBoundingClientRect(), c = getComputedStyle(e); return { cls: String(e.className.baseVal ?? e.className), w: r.width, h: r.height, x: r.left, y: r.top, bw: c.borderTopWidth, bc: c.borderTopColor, pad: c.paddingTop, rad: c.borderTopLeftRadius }; })`), null, 1));
      return;
    }
    const failures = []; let ran = 0;
    for (const spec of require('./spec.cjs')) {
      if (flag('--only') && spec.name !== flag('--only')) continue;
      const next=JSON.stringify(spec.viewport||{width:1280,height:960});if(next!==viewport){await browser.close();browser=await launch(chrome,spec.viewport);viewport=next;}
      ran++; const measured = {}, shots = {};
      for (const side of ['mock', 'app']) {
        await visit(browser, side, side === 'mock' ? spec.mock : spec.app);
        const s = spec[side]; measured[side] = await browser.eval(MEASURE(s.selector, s.index || 0, spec.props, s.mark));
        if (!measured[side]) { failures.push(`${spec.name}: ${side} element not found (${s.selector})`); continue; }
        const m = measured[side]; shots[side] = await browser.shot({ x: m.x, y: m.y, width: m.width, height: m.height });
        fs.writeFileSync(path.join(out, `${spec.name}.${side}.png`), shots[side]);
      }
      if (!measured.mock || !measured.app) continue;
      const tol = spec.tol ?? 0.5, cmp = (label, a, b) => { for (const k of Object.keys(a)) {
        const x = a[k], y = b[k], num = typeof x === 'number'; if (num ? Math.abs(x - y) > (k === 'x' || k === 'y' ? Infinity : tol) : String(x) !== String(y)) failures.push(`${spec.name}${label}.${k}: mock ${x} app ${y}`); } };
      cmp('', measured.mock, measured.app);
      if (shots.mock && shots.app && spec.pixel !== false) { const d = diffRatio(decode(shots.mock), decode(shots.app)); console.log(`  ${spec.name}: ${(d.ratio * 100).toFixed(2)}% of pixels differ`); if (d.ratio > (spec.maxPixelDiff ?? 0.02)) failures.push(`${spec.name}: ${(d.ratio * 100).toFixed(1)}% of pixels differ${d.sizeMismatch ? ' (size ' + d.sizeMismatch.join(',') + ')' : ''}`); }
    }
    console.log(`design parity: ${ran} element(s), ${failures.length} difference(s); captures in ${out}`);
    failures.forEach(f => console.log('  - ' + f));
    process.exitCode = failures.length ? 1 : 0;
  } finally { await browser.close(); }
})().catch(e => { console.error(e); process.exit(2); });
