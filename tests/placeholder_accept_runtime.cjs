const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const { pathToFileURL } = require('node:url');
const { findChrome, launch } = require('./design_parity/cdp.cjs');

(async () => {
  const chrome = findChrome();
  if (!chrome) { process.exitCode = 77; return; }
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-placeholder-'));
  let browser;
  try {
    const app = path.join(dir, 'app.html');
    execFileSync(process.execPath, ['tests/ui_sim/build.cjs', process.cwd(), app]);
    browser = await launch(chrome, { width: 1280, height: 900 });
    await browser.goto(pathToFileURL(app).href);
    const result = await browser.eval(`(async () => {
      const q = id => document.getElementById('start-' + id);
      for (let i = 0; q('chat-button').disabled && i < 100; i++) {
        await new Promise(resolve => setTimeout(resolve, 30));
      }
      if (q('chat').hidden) q('chat-button').click();
      const input = q('chat-input');
      const press = (field, key, options = {}) => !field.dispatchEvent(new KeyboardEvent('keydown', {
        key, bubbles: true, cancelable: true, ...options
      }));
      const reset = () => {
        input.blur();
        // Headless Chrome reports document.hasFocus() as false and suppresses focusout.
        if (!document.hasFocus()) input.dispatchEvent(new FocusEvent('focusout', { bubbles: true }));
        input.value = '';
        input.focus();
      };
      const out = { focused: document.activeElement === input };
      for (const [lang, key] of [['en', 'Tab'], ['ko', 'ArrowRight']]) {
        AC_I18N.set(lang);
        reset();
        const accepted = press(input, key);
        out[lang] = accepted && input.value === input.placeholder && !!input.value
          && input.selectionStart === input.value.length && input.selectionEnd === input.value.length;
        out.typedUntouched = !press(input, 'Tab') && input.value === input.placeholder;
      }
      reset();
      out.shiftTab = !press(input, 'Tab', { shiftKey: true }) && input.value === '';
      reset();
      out.ime = !press(input, 'ArrowRight', { isComposing: true }) && input.value === '';
      out.typing = !press(input, 'a') && input.value === ''
        && getComputedStyle(input, '::placeholder').color === 'rgba(0, 0, 0, 0)'
        && !press(input, 'ArrowRight');
      reset();
      out.restored = press(input, 'ArrowRight') && input.value === input.placeholder;
      const hint = document.createElement('input');
      hint.placeholder = 'Example only';
      document.body.append(hint);
      out.normalHint = !press(hint, 'Tab') && hint.value === '';
      hint.setAttribute('data-placeholder-suggestion', '');
      out.shared = press(hint, 'ArrowRight') && hint.value === 'Example only';
      out.noSend = !SIM.log.some(e => e.kind === 'chat');
      return out;
    })()`);
    for (const [scenario, passed] of Object.entries(result)) assert.equal(passed, true, scenario);
    console.log('Placeholder suggestions: acceptance, continuation, editing, focus and IME passed.');
  } finally {
    try { await browser?.close(); } finally { fs.rmSync(dir, { recursive: true, force: true }); }
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
