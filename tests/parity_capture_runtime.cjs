// #420: unstable evidence and an unsettled reload must never look like a passed comparison.
const assert = require('node:assert/strict');
const fs = require('node:fs'), path = require('node:path'), os = require('node:os');
const { spawnSync } = require('node:child_process');
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-parity-capture-'));
try {
  const hook = path.join(dir, 'hook.cjs');
  fs.writeFileSync(hook, `
const Module = require('node:module'), path = require('node:path'), load = Module._load;
const mode = process.env.PARITY_CAPTURE_CASE;
let captureChecks = 0, ready = true;
process.on('exit', () => console.log('capture-checks=' + captureChecks));
global.setTimeout = (fn, _delay, ...args) => setImmediate(fn, ...args);
Module._load = function(name, parent, ...rest) {
  if (path.basename(parent?.filename || '') === 'parity.cjs') {
    if (name === './spec.cjs') return [{name:'capture-fixture',props:[],mock:{selector:'#mock'},app:{selector:'#app'}}];
    if (name === './cdp.cjs') return {
      findChrome: () => 'fixture',
      launch: async () => ({
        goto: async () => { ready = true; },
        viewport: async () => {}, close: async () => {},
        eval: async expr => {
          if (mode === 'reload' && expr.includes('location.reload()')) { ready = false; return; }
          if (!ready) throw Error('evaluation before the new document is ready');
          if (expr === "!!document.querySelector('[data-parity-target]')") {
            captureChecks++;
            return mode === 'never' ? false : mode === 'late' ? captureChecks % 5 === 0 : true;
          }
          return {x:0,y:0,width:2,height:2};
        },
        // An owned 2x2 white Chrome screenshot: equal pixels cannot establish capture stability.
        shot: async () => Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAIAAAD91JpzAAAAE0lEQVR4nGL5//8/AwMDEwMYAAAAAP//aYxtrAAAAAZJREFUAwAkMAMEkRkhTQAAAABJRU5ErkJggg==','base64')
      })
    };
  }
  return load.call(this, name, parent, ...rest);
};
`);
  for (const [mode, status, checks] of [['never',2,5],['late',0,10],['ready',0,2],['reload',0,2]]) {
    const result = spawnSync(process.execPath,
      ['--require',hook,'tests/design_parity/parity.cjs','--out',path.join(dir,mode)],
      {env:{...process.env,PARITY_CAPTURE_CASE:mode,PARITY_WIDTH:'840',PARITY_THEME:'light'},encoding:'utf8',timeout:15000});
    assert.equal(result.status, status, mode + ': ' + result.stdout + result.stderr);
    assert.match(result.stdout, new RegExp('capture-checks=' + checks));
    if (mode === 'never') assert.match(result.stderr, /five capture attempts/);
  }
  console.log('Capture stability, fifth-attempt success and awaited navigation are enforced.');
} finally {
  fs.rmSync(dir, {recursive:true,force:true});
}
