// GitHub #414: compare the app UI with the approved design mock (docs/design/approved-goal-ui/preview.html)
// at the same viewport, element by element: geometry and computed style within a pixel tolerance, plus the
// share of differing pixels in the element's screenshot crop. Usage:
//   [PARITY_THEME=dark] [PARITY_WIDTH=840] node tests/design_parity/parity.cjs [--out dir] [--only name] | --probe mock|app <selector> [url-suffix]
// (--probe app honours PROBE_SETUP / PROBE_RUN: simulator state to set, and a script to run, before measuring)
// Exit 0 = all within tolerance, 1 = a difference, 77 = skipped (no Chrome). Spec: spec.cjs.
const fs = require('fs'), path = require('path'), { execFileSync } = require('child_process');
const { findChrome, launch } = require('./cdp.cjs'), { decode, diffRatio } = require('./png.cjs');
const { describe } = require('./diagnostics.cjs');
const { pathToFileURL } = require('url'), repo = path.resolve(__dirname, '../..'), MOCK = pathToFileURL(path.join(repo, 'docs/design/approved-goal-ui/preview.html')).href;
const THEME = process.env.PARITY_THEME === 'dark' ? 'dark' : 'light', WIDTH = Number(process.env.PARITY_WIDTH) || 1280;
const diffGrid = (a, b) => { const g = {}; for (let y = 0; y < a.height; y++) for (let x = 0; x < a.width; x++) { const i = (y * a.width + x) * 4; if ([0, 1, 2, 3].some(k => a.data[i + k] !== b.data[i + k])) { const c = `${y >> 3},${x >> 3}`; g[c] = (g[c] || 0) + 1; } } return g; };
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
const MEASURE = (sel, index, props, mark, rel) => `(() => { const e = [...document.querySelectorAll(${JSON.stringify(sel)})].filter(x => x.getBoundingClientRect().width > 0)[${index}]; if (!e) return null;
  const r = e.getBoundingClientRect(), c = getComputedStyle(e), o = { x: r.left, y: r.top, width: r.width, height: r.height };
  for (const p of ${JSON.stringify(props)}) o[p] = c[p];
  const k = ${JSON.stringify(mark || null)} && e.querySelector(${JSON.stringify(mark || 'x')});
  if (${JSON.stringify(mark || null)} === 'content') o.mark = r.width - 2 * (parseFloat(c.borderLeftWidth) + parseFloat(c.paddingLeft)); else if (k) o.mark = k.getBoundingClientRect().width;
  const q = ${JSON.stringify(rel || null)} && [...document.querySelectorAll(${JSON.stringify(rel || 'x')})].filter(x => x.getBoundingClientRect().width > 0)[0];
  if (q) { const b = q.getBoundingClientRect(); o.dy = r.top - b.bottom; o.dc = r.top + r.height / 2 - (b.top + b.height / 2); o.dt = r.top - b.top; o.dl = r.left - b.left; }
  return o; })()`;

async function visit(browser, side, spec) {
  if (side === 'mock') {
    await browser.goto('about:blank'); await browser.goto(MOCK + (spec.url || '?theme=light#0').replace('theme=light', 'theme=' + THEME));
    // The preview canvas has padding and a frame border; compare the same native client width.
    await browser.eval(`(() => { const frame=document.querySelector('.win'); if(!frame)throw Error('reference window missing');
      frame.style.maxWidth='none'; frame.style.width=(innerWidth+frame.offsetWidth-frame.clientWidth)+'px';
      if(frame.clientWidth!==innerWidth)throw Error('reference client width does not match viewport'); })()`);
  } else {
    await browser.goto(APP); await browser.eval(`sessionStorage.setItem('SIM_PRESET', ${JSON.stringify(PRESET + (spec.setup || ''))})`);
    await browser.goto(APP);
  }
  if (side === 'app') await browser.eval(`document.documentElement.dataset.theme = ${JSON.stringify(THEME)}`);
  // GitHub #482: the control a run script clicks may render a moment later (quota buttons); retry until it exists.
  if (spec.run) for (let i = 0; ; i++) { try { await browser.eval(spec.run); break; } catch (e) { if (i >= 50) throw e; await new Promise(r => setTimeout(r, 100)); } }
  await new Promise(r => setTimeout(r, 600));
  // Pulses and slides are checked by their own tests; freeze them so the pixels are comparable.
  await browser.eval("document.head.append(Object.assign(document.createElement('style'), { textContent: '*,*::before,*::after{animation:none!important;transition:none!important}' }))");
  await new Promise(r => setTimeout(r, 150));
}

(async () => {
  const browser = await launch(chrome, { width: WIDTH });
  try {
    if (argv[0] === '--probe') {
      await visit(browser, argv[1], { url: argv[3], setup: process.env.PROBE_SETUP, run: process.env.PROBE_RUN });
      console.log(JSON.stringify(await browser.eval(`[...document.querySelectorAll(${JSON.stringify(argv[2])})].slice(0, 6).map(e => { const r = e.getBoundingClientRect(), c = getComputedStyle(e); return { cls: String(e.className.baseVal ?? e.className), w: r.width, h: r.height, x: r.left, y: r.top, bw: c.borderTopWidth, bc: c.borderTopColor, pad: c.paddingTop, rad: c.borderTopLeftRadius }; })`), null, 1));
      return;
    }
    const failures = [], specs = require('./spec.cjs').filter(x => (!flag('--only') || x.name === flag('--only')) && (!x.widths || x.widths.includes(WIDTH))), ran = specs.length;
    // Measure grouped by page state so each state is loaded once (the run is dominated by page loads).
    const viewport = x => Array.isArray(x.viewport) ? x.viewport : [x.viewport?.width || WIDTH, x.viewport?.height || 960];
    const key = (side, x) => JSON.stringify([side, viewport(x), x[side].url || '', x[side].setup || '', x[side].run || '']);
    const jobs = []; for (const x of specs) if (!x.assert) for (const side of ['mock', 'app']) jobs.push({ x, side });
    jobs.sort((p, q) => (key(p.side, p.x) < key(q.side, q.x) ? -1 : key(p.side, p.x) > key(q.side, q.x) ? 1 : 0));
    const got = {}; let current = null;
    for (const { x, side } of jobs) {
      const k = key(side, x); if (k !== current) { await browser.viewport(...viewport(x)); await visit(browser, side, x[side]); current = k; }
      const s = x[side];
      if (x.hover) { const c = await browser.eval(MEASURE(s.selector, s.index || 0, [], null)); if (c) { await browser.hover(c.x + c.width / 2, c.y + c.height / 2); await new Promise(r => setTimeout(r, 150)); } }
      if (x.focus) { await browser.tab(); await browser.eval(`[...document.querySelectorAll(${JSON.stringify(s.selector)})].filter(e => e.getBoundingClientRect().width > 0)[${s.index || 0}]?.focus()`); }
      // GitHub #482: raster the same pixels every run - images decoded, fonts loaded, two frames painted.
      await browser.eval("(async()=>{await document.fonts.ready;await Promise.all([...document.images].map(i=>i.decode().catch(()=>{})));await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));})()");
      const m = await browser.eval(MEASURE(s.selector, s.index || 0, x.props || [], s.mark, s.rel));
      if (x.focus) await browser.eval('document.activeElement && document.activeElement.blur()');
      if (x.hover) await browser.hover(0, 0);
      got[x.name] = got[x.name] || { shots: {} }; got[x.name][side] = m;
      got[x.name].diagnostics ||= {};got[x.name].diagnostics[side]=await describe(browser,s.selector,s.index||0);
      if (!m) { failures.push(`${x.name}: ${side} element not found (${s.selector})`); continue; }
      // Chrome rasterises the same drawing a little differently at different page positions and on different backdrops, so each element is
      // captured alone: lifted to the same fixed spot on a white page, at its own measured size, with everything else hidden.
      for (let attempt = 0; attempt < 5; attempt++) {
        await browser.eval(`(() => { const e = [...document.querySelectorAll(${JSON.stringify(s.selector)})].filter(x => x.getBoundingClientRect().width > 0)[${s.index || 0}];
          if (${x.clone ? 'true' : 'false'}) {
            // GitHub #482: an element that Windows rasterises differently inside the two pages is compared as a copy with its
            // computed styles inlined, so both sides draw from the same context.
            const inline = (s, d) => { const c = getComputedStyle(s); for (const k of c) d.style.setProperty(k, c.getPropertyValue(k)); [...s.children].forEach((ch, i) => inline(ch, d.children[i])); };
            const copy = e.cloneNode(true); inline(e, copy); copy.dataset.parityClone = '1'; copy.dataset.parityTarget = '1';
            copy.style.cssText += ';position:fixed;left:100px;top:100px;margin:0;box-sizing:border-box;width:${m.width}px;height:${m.height}px;transform:none;translate:none;z-index:99999';
            document.body.append(copy);
          } else {
          e.dataset.parityStyle = e.getAttribute('style') || ''; e.dataset.parityTarget = '1';
          e.style.cssText += '${x.inPlace ? '' : `;position:fixed;left:100px;top:100px;margin:0;box-sizing:border-box;width:${m.width}px;height:${m.height}px;transform:none;translate:none;z-index:99999`}';
          }
          document.head.append(Object.assign(document.createElement('style'), { id: 'parity-solo', textContent: 'html,body{background:#fff!important}*{visibility:hidden!important}[data-parity-target],[data-parity-target] *{visibility:visible!important}' + (${x.flat ? 'true' : 'false'} ? '[data-parity-target]{box-shadow:none!important;overflow:visible!important}[data-parity-target]::backdrop{background:transparent!important}' : '') })); })()`);
        got[x.name].shots[side] = await browser.shot(x.inPlace ? { x: m.x, y: m.y, width: m.width, height: m.height } : { x: 100, y: 100, width: m.width, height: m.height });
        const kept = await browser.eval("!!document.querySelector('[data-parity-target]')");
        await browser.eval(`(() => { document.getElementById('parity-solo')?.remove(); const e = document.querySelector('[data-parity-target]'); if (!e) return; if (e.dataset.parityClone) { e.remove(); return; } e.setAttribute('style', e.dataset.parityStyle); if (!e.dataset.parityStyle) e.removeAttribute('style'); delete e.dataset.parityStyle; delete e.dataset.parityTarget; })()`);
        if (kept) break;
        if (attempt === 4) throw new Error(`${x.name}: ${side} changed during all five capture attempts`);
      }
      fs.writeFileSync(path.join(out, `${x.name}.${side}.png`), got[x.name].shots[side]);
    }
    for (const spec of specs) {
      if (spec.assert) {
      // behaviour check on the app only: `assert` is evaluated after `run` and must be true
        await browser.viewport(...viewport(spec));
        await visit(browser, 'app', spec.app); const ok = await browser.eval(spec.assert);
        if (spec.viewport) await browser.viewport(WIDTH, 960);
        if (ok !== true) failures.push(`${spec.name}: assertion failed (${spec.assert})`); continue;
      }
      const measured = got[spec.name], shots = measured.shots;
      if (!measured.mock || !measured.app) continue;
      const tol = 0.01, cmp = (label, a, b) => { for (const k of Object.keys(a)) { if (k === 'shots' || (spec.keys && !spec.keys.includes(k))) continue;
        const x = a[k], y = b[k], num = typeof x === 'number'; if (num ? Math.abs(x - y) > (k === 'x' || k === 'y' ? Infinity : tol) : String(x) !== String(y)) failures.push(`${spec.name}${label}.${k}: mock ${x} app ${y}`); } };
      cmp('', measured.mock, measured.app);
      if (shots.mock && shots.app && spec.pixel !== false) { const d = diffRatio(decode(shots.mock), decode(shots.app)); console.log(`  ${spec.name}: ${(d.ratio * 100).toFixed(2)}% of pixels differ`); if (d.ratio > (spec.tolerance || 0)) { failures.push(`${spec.name}: ${(d.ratio * 100).toFixed(1)}% of pixels differ${d.sizeMismatch ? ' (size ' + d.sizeMismatch.join(',') + ')' : ''}`); if (!d.sizeMismatch) console.log(`  ${spec.name} differing-pixel grid (8px cells, row,col:count): ${JSON.stringify(diffGrid(decode(shots.mock), decode(shots.app)))}`); } }
    }
    console.log(`design parity: ${ran} element(s), ${failures.length} difference(s); captures in ${out}`);
    failures.forEach(f => console.log('  - ' + f));
    const bad=new Set(failures.map(f=>f.split(':')[0].split('.')[0]));
    for(const name of bad)if(got[name]?.diagnostics){const info=JSON.stringify(got[name].diagnostics,null,2);fs.writeFileSync(path.join(out,name+'.diagnostics.json'),info);console.log('diagnostics '+name+': '+info);}

    process.exitCode = failures.length ? 1 : 0;
  } finally { await browser.close(); }
})().catch(e => { console.error(e); process.exit(2); });
