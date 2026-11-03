// GitHub #398 / DD-222: the centered result popup of a completed goal. The helpers are pure; openResult reads
// the existing evidence and task requests, so no request kind or backend change is involved.
const resultFileHeader = /^diff --git a\/(.+?) b\//;
function resultFiles(diff) {
  const files = []; let current;
  for (const line of String(diff || '').split('\n')) {
    const header = resultFileHeader.exec(line);
    if (header) { current = { path: header[1], add: 0, del: 0 }; files.push(current); }
    else if (current && line.startsWith('+') && !line.startsWith('+++')) current.add++;
    else if (current && line.startsWith('-') && !line.startsWith('---')) current.del++;
  }
  return files;
}
function resultTests(outputs) { const list = outputs || []; return { passed: list.filter(t => t.exit_code === 0 && !t.timed_out).length, total: list.length }; }
// Opens once when a watched run completes; a run that was already finished when the Room opened does not pop up.
function resultTransition(prev, next) { return next === 'completed' && !!prev && prev !== 'completed'; }
function resultLessons(details) {
  const out = [];
  for (const d of details || []) { for (const l of d.step_lessons || []) out.push(l.response); for (const k of d.knowledge || []) out.push(k.lesson?.response); }
  return [...new Set(out.filter(Boolean))];
}
let resultState;
function watchResult(snap) {
  const state = snap.report.state;
  $('show-result').hidden = state !== 'completed';
  if (resultTransition(resultState, state)) openResult().catch(e => notice(e.message, true));
  resultState = state;
}
async function openResult() {
  const r = snapshot.report, review = await request({ kind: 'evidence' });
  const details = await Promise.all(r.tasks.map((_, index) => request({ kind: 'task', index }).catch(() => ({}))));
  const files = resultFiles(review.diff), tests = resultTests(review.test_output), lessons = resultLessons(details);
  const row = (a, b) => { const el = node('li'); el.append(node('span', a), node('span', b, 'muted')); return el; };
  AC_I18N.bind($('result-title'), () => acMessage('room.result_title', { value1: r.plan.goal }));
  AC_I18N.bind($('result-target'), () => acMessage('room.result_target', { value1: r.branch, value2: (review.report.candidate?.commit || '').slice(0, 12), value3: r.repo }));
  const metric = (label, value) => { const el = node('div', undefined, 'result-metric'); el.append(node('span', label, 'muted'), node('strong', String(value))); return el; };
  AC_I18N.replace($('result-metrics'), metric(() => acMessage('room.result_metric_files'), files.length), metric(() => acMessage('room.result_metric_tests'), `${tests.passed}/${tests.total}`), metric(() => acMessage('room.result_metric_lessons'), lessons.length));
  AC_I18N.replace($('result-files'), ...files.map(f => row(f.path, `+${f.add} / −${f.del}`)));
  AC_I18N.replace($('result-tests'), ...review.test_output.map(t => row(t.task, () => acMessage(t.exit_code === 0 && !t.timed_out ? 'room.result_test_passed' : 'room.result_test_failed'))));
  AC_I18N.replace($('result-lessons'), ...(lessons.length ? lessons.map(l => node('li', l)) : [node('li', () => acMessage('room.result_no_lessons'), 'muted')]));
  if (!$('result-dialog').open) $('result-dialog').showModal();
}
