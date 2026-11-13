// Shared existing fixture constants; no change to the generated comparison entries.
const BOX = ['borderTopWidth', 'borderTopLeftRadius'];
// Include the badge outside the avatar border; wrapping the real node preserves its actual pseudo-element.
const attentionBox = selector => `(() => { const e=document.querySelector(${JSON.stringify(selector)}),box=document.createElement('span');box.id='attention-probe';box.style.cssText='display:inline-flex;flex:none;padding:6px;background:transparent';e.replaceWith(box);box.append(e); })()`;
// The mock has no disabled step; its tabs are all reachable, so the app is compared with a run available (disabled steps dim by design).
const STEP_ONE = "document.querySelector('#start-step-tabs [role=tab]').click();";
const INSPECTOR_OPEN = "(async()=>{document.querySelectorAll('#start-step-tabs [role=tab]')[3].click();await new Promise(r=>setTimeout(r,900));document.querySelector('.inspector-switch').click();})()";
const STEPS_ENABLED = "SIM.home.run_available=true;SIM.home.run_state='running';";
// The reference window has a decorative border and outer margin; compare equal inner widths.
const STEP_REFERENCE_WIDTH = "document.querySelector('.win').style.cssText='width:calc(100vw + 2px);max-width:none'";
module.exports={BOX,attentionBox,STEP_ONE,INSPECTOR_OPEN,STEPS_ENABLED,STEP_REFERENCE_WIDTH};
