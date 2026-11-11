// GitHub #414: a tiny Chrome DevTools Protocol driver for the design parity check (no npm dependency).
// Node's global WebSocket and a local headless Chrome are enough.
const { stopChrome } = require('./chrome_process.cjs');
const { spawn } = require('child_process'), fs = require('fs'), os = require('os'), path = require('path');

function findChrome() {
  const found = [process.env.CHROME, '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    'C:/Program Files/Google/Chrome/Application/chrome.exe', 'C:/Program Files (x86)/Google/Chrome/Application/chrome.exe',
    '/usr/bin/google-chrome', '/usr/bin/chromium', '/usr/bin/chromium-browser'].filter(Boolean).find(p => fs.existsSync(p));
  return found || null;
}

// Scale 2 matches a Retina display, where the 1.5px status border is real (at 1x Chrome rounds it to 1px).
async function launch(chrome, { width = 1280, height = 960, scale = 2 } = {}) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-parity-'));
  let proc, ws;
  try {
    proc = spawn(chrome, ['--headless=new', '--remote-debugging-port=0', `--user-data-dir=${dir}`, '--no-first-run',
      `--force-device-scale-factor=${scale}`, '--hide-scrollbars', '--disable-lcd-text', '--font-render-hinting=none', '--disable-gpu', '--force-color-profile=srgb', '--allow-file-access-from-files', `--window-size=${width},${height}`, 'about:blank'], { stdio: 'ignore' });
    let spawnError;
    proc.once('error', error => { spawnError = error; });
    const portFile = path.join(dir, 'DevToolsActivePort');
    for (let i = 0; i < 400 && !fs.existsSync(portFile); i++) {
      if (spawnError) throw spawnError;
      if (proc.exitCode !== null || proc.signalCode !== null) throw new Error('Chrome exited before DevTools started: ' + (proc.exitCode ?? proc.signalCode));
      await new Promise(r => setTimeout(r, 100));
    }
    if (!fs.existsSync(portFile)) throw new Error('Chrome did not start');
    // Windows: Chrome may still hold the file open for a moment (EBUSY), so retry the read.
    let port; for (let i = 0; !port; i++) { try { port = fs.readFileSync(portFile, 'utf8').split('\n')[0]; } catch (e) { if (i >= 50 || !['EBUSY', 'EPERM', 'EACCES'].includes(e.code)) throw e; await new Promise(r => setTimeout(r, 100)); } }
    const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
    ws = new WebSocket(targets.find(t => t.type === 'page').webSocketDebuggerUrl);
    await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
    let id = 0; const waiting = new Map(), events = [];
    ws.onmessage = m => { const d = JSON.parse(m.data); if (d.id && waiting.has(d.id)) { const { res, rej } = waiting.get(d.id); waiting.delete(d.id); d.error ? rej(new Error(d.error.message)) : res(d.result); } else events.push(d); };
    // GitHub #669: a DevTools call that gets no answer (a stuck page or renderer) fails by name instead of hanging the whole job.
    const send = (method, params = {}) => new Promise((res, rej) => {
      const n = ++id, timer = setTimeout(() => { waiting.delete(n); rej(new Error(`DevTools call ${method} got no answer in 90s`)); }, 90000);
      waiting.set(n, { res: v => { clearTimeout(timer); res(v); }, rej: e => { clearTimeout(timer); rej(e); } });
      ws.send(JSON.stringify({ id: n, method, params }));
    });
    await send('Page.enable'); await send('Page.setLifecycleEventsEnabled', { enabled: true }); await send('Runtime.enable');
    await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: scale, mobile: false });
    // GitHub #527: a hover is measured after two rendered frames and every running transition has ended, not after a fixed 60 ms.
    const api = {
      // Waits for the load of the document this navigation created (matched by loaderId), so a slow response never hands back a
      // half-created page; a same-document navigation (hash only) has no loader and no load event.
      async goto(url) {
        events.length = 0; const nav = await send('Page.navigate', { url });
        if (nav.errorText) throw new Error(`Navigation to ${url} failed: ${nav.errorText}`);
        if (nav.loaderId) {
          const loaded = () => events.some(e => e.method === 'Page.lifecycleEvent' && e.params.name === 'load' && e.params.loaderId === nav.loaderId);
          for (let i = 0; i < 600 && !loaded(); i++) await new Promise(r => setTimeout(r, 100));
          if (!loaded()) throw new Error(`Page did not load within 60 s: ${url}`);
        }
        await new Promise(r => setTimeout(r, 400));
      },
      async eval(expr) { const r = await send('Runtime.evaluate', { expression: expr, awaitPromise: true, returnByValue: true }); if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description || r.exceptionDetails.text); return r.result.value; },
      async hover(x, y) { await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y }); await api.eval(`new Promise(done => requestAnimationFrame(() => requestAnimationFrame(async () => { await Promise.race([Promise.all(document.getAnimations().map(a => a.finished.catch(() => {}))), new Promise(r => setTimeout(r, 2000))]); done(); })))`); },
      async tab() { for (const type of ['keyDown', 'keyUp']) await send('Input.dispatchKeyEvent', { type, key: 'Tab', code: 'Tab', windowsVirtualKeyCode: 9 }); },
      async fonts(selector, index = 0) {
        await send('DOM.enable'); await send('CSS.enable'); await send('DOM.getDocument');
        const r = await send('Runtime.evaluate', { expression: `[...document.querySelectorAll(${JSON.stringify(selector)})].filter(e=>e.getBoundingClientRect().width>0)[${index}]`, returnByValue: false });
        if (!r.result.objectId) return [];
        try {
          const n = await send('DOM.requestNode', { objectId: r.result.objectId });
          return (await send('CSS.getPlatformFontsForNode', { nodeId: n.nodeId })).fonts;
        } finally { await send('Runtime.releaseObject', { objectId: r.result.objectId }); }
      },
      async viewport(w, h) { await send('Emulation.setDeviceMetricsOverride', { width: w, height: h, deviceScaleFactor: scale, mobile: false }); },
      async shot(clip) { const r = await send('Page.captureScreenshot', { format: 'png', clip: clip && { ...clip, scale: 1 } }); return Buffer.from(r.data, 'base64'); },
      async close() { try { ws.close(); } catch {} await stopChrome(proc); fs.rmSync(dir, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 }); },
    };
    return api;
  } catch (error) {
    try { ws?.close(); } catch {}
    if (Number.isInteger(proc?.pid) && proc.pid > 0) await stopChrome(proc);
    fs.rmSync(dir, { recursive: true, force: true, maxRetries: 10, retryDelay: 200 });
    throw error;
  }
}
module.exports = { findChrome, launch };
