// #420: compare equal logical client widths, not a padded preview canvas with a full app window.
const assert = require('node:assert/strict');
const fs = require('node:fs'), path = require('node:path'), os = require('node:os');
const {spawnSync} = require('node:child_process');
const {findChrome} = require('./design_parity/cdp.cjs');
if (!findChrome()) { console.log('Reference client check skipped: no Chrome'); process.exit(77); }
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ac-parity-reference-'));
try {
  for (const theme of ['light','dark']) for (const width of [840,1280]) {
    const measured = {};
    for (const [side,selector] of [['app','#start-titlebar'],['mock','.titlebar']]) {
      const result = spawnSync(process.execPath,
        ['tests/design_parity/parity.cjs','--probe',side,selector,'?theme=light#0','--out',path.join(dir,theme+'-'+width+'-'+side)],
        {env:{...process.env,PARITY_THEME:theme,PARITY_WIDTH:String(width)},encoding:'utf8',timeout:30000});
      assert.equal(result.status,0,side+': '+result.stderr+result.stdout);
      const visible = JSON.parse(result.stdout).find(value=>value.w>0);
      assert(visible,side+' has a visible titlebar');
      measured[side] = visible.w;
    }
    assert.equal(measured.app,width,'app client width follows emulated viewport');
    assert.equal(measured.mock,measured.app,theme+'/'+width+' compares the same logical window client width');
    console.log(theme,width,'reference and app client widths match');
  }
} finally { fs.rmSync(dir,{recursive:true,force:true}); }
