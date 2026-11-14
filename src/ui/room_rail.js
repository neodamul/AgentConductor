// GitHub #675: the task rail reads like the mock's outline: current Phase and Task, tasks done, elapsed time, the Phase and its task rows.
const OUTLINE_STATE = { ok: 'completed', run: 'running', wait: 'waiting', bad: 'error' };
// The mock's badge colours: done and running are green, waiting amber, failed red.
function outlineState(status) { return OUTLINE_STATE[statusTone(status)] || 'waiting'; }
function chevron() {
  const ns = 'http://www.w3.org/2000/svg', svg = document.createElementNS(ns, 'svg'), path = document.createElementNS(ns, 'path');
  for (const [k, v] of [['viewBox', '0 0 24 24'], ['width', '12'], ['height', '12'], ['fill', 'none'], ['stroke', 'currentColor'], ['stroke-width', '1.6'], ['stroke-linecap', 'round'], ['stroke-linejoin', 'round'], ['aria-hidden', 'true'], ['class', 'outline-open-icon']]) svg.setAttribute(k, v);
  path.setAttribute('d', 'M9 5l7 7-7 7'); svg.append(path); return svg;
}
function renderRail(r, selected) {
  const ph = snapshot.phase, list = ph?.phases || [], pi = list.findIndex(x => x.id === ph?.phase), finished = r.tasks.filter(t => t.status === 'completed').length;
  AC_I18N.bind($('rail-current'), () => acMessage("room.rail_current", { value1: pi < 0 ? 1 : pi + 1, value2: pi < 0 ? 1 : list.length, value3: Math.min(r.tasks.length, selected + 1), value4: r.tasks.length }));
  AC_I18N.bind($('rail-done'), () => acMessage("room.rail_done", { value1: finished, value2: r.tasks.length }));
  AC_I18N.bind($('rail-phase-name'), () => (pi < 0 ? '1. ' + taskLine(r.plan.goal) : (pi + 1) + '. ' + (list[pi].name || ph.phase)));
  AC_I18N.bind($('rail-phase-brief'), () => (ph?.description || '')); $('rail-phase-brief').hidden = !ph?.description;
}
// The run's elapsed time: from the first task start to now, frozen at the last finish once every task is done.
function railElapsed(r, now) {
  const starts = r.tasks.map(t => Number(t.started_ms)).filter(Boolean), ends = r.tasks.map(t => Number(t.finished_ms)).filter(Boolean);
  if (!starts.length) return '';
  const done = r.tasks.every(t => t.status === 'completed'), sec = Math.max(0, Math.floor(((done && ends.length ? Math.max(...ends) : now) - Math.min(...starts)) / 1000));
  const h = Math.floor(sec / 3600), m = String(Math.floor(sec % 3600 / 60)).padStart(2, '0'), s = String(sec % 60).padStart(2, '0');
  return acMessage("room.rail_elapsed", { value1: h ? `${h}:${m}:${s}` : `${m}:${s}` });
}
// The line under the title (the mock's goal-info row): the integration result while a candidate waits for approval.
function renderGoalInfo(r) {
  const failed = r.tests.filter(x => !(x.passed??(x.exit_code===0&&!x.timed_out))), text = isVerifying(r) ? acMessage('room.goal_info_verifying', { value1: r.tests.length, value2: r.tasks.length }) : r.state === 'waiting_approval' && r.tests.length ? (failed.length ? acMessage('room.goal_info_tests_failed', { value1: failed.length }) : acMessage('room.goal_info_tests_passed')) : r.state==='approved'?acMessage('room.approved_merging_remains_a_separate_action'):r.state==='completed'?completedStep(document.getElementById('startup-home')):snapshot.phase_continuation?.unchanged?acMessage('room.phase_unchanged_started',{value1:snapshot.phase_continuation.number,value2:snapshot.phase_continuation.total,value3:snapshot.phase_continuation.previous}):'';
  AC_I18N.bind(document.querySelector('#run-goal-info .goal-info-content'), () => text);
}

function completedStep(home) {
 const ph=snapshot.phase,phases=ph?.phases||[],i=phases.findIndex(x=>x.id===ph?.phase),n=phases.findIndex(x=>x.id===ph?.current_phase);
 if(i<0)return home?acMessage('room.this_run_is_complete_return_to_the_workspace'):acMessage('room.this_run_is_complete');
 if(n<0)return acMessage('room.phase_of_is_complete_and_with_it_every',{value1:i+1,value2:phases.length});
 return acMessage('room.phase_of_is_complete_phase_is_next_go',{value1:i+1,value2:phases.length,value3:phases[i].name||ph.phase,value4:n+1,value5:phases[n].name||ph.current_phase,value6:n+1});
}
