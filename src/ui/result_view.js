// GitHub #398 / #402 / DD-222: the centered result popup of a completed goal. The helpers are pure; show() fills the
// shared #result-dialog, so the Agent Room (automatic and Result button) and the Goal list use the same popup.
window.acResultView = (() => {
  const $ = id => document.getElementById(id);
  const node = (tag, text, cls) => { const el = document.createElement(tag); if (text !== undefined) AC_I18N.label(el, text); if (cls) el.className = cls; return el; };
  const fileHeader = /^diff --git a\/(.+?) b\//;
  function files(diff) {
    const list = []; let current;
    for (const line of String(diff || '').split('\n')) {
      const header = fileHeader.exec(line);
      if (header) { current = { path: header[1], add: 0, del: 0 }; list.push(current); }
      else if (current && line.startsWith('+') && !line.startsWith('+++')) current.add++;
      else if (current && line.startsWith('-') && !line.startsWith('---')) current.del++;
    }
    return list;
  }
  function tests(outputs) { const list = outputs || []; return { passed: list.filter(t => t.exit_code === 0 && !t.timed_out).length, total: list.length }; }
  function lessons(details) {
    const out = [];
    for (const d of details || []) { for (const l of d.step_lessons || []) out.push(l.response); for (const k of d.knowledge || []) out.push(k.lesson?.response); }
    return [...new Set(out.filter(Boolean))];
  }
  // data: { goal, branch, repo, commit, diff, tests (test_output list), details (per-task data) }
  function show(data) {
    const changed = files(data.diff), outputs = data.tests || [], summary = tests(outputs), notes = lessons(data.details);
    const row = (a, b) => { const el = node('li'); el.append(node('span', a), node('span', b, 'muted')); return el; };
    AC_I18N.bind($('result-title'), () => acMessage('room.result_title', { value1: data.goal }));
    AC_I18N.bind($('result-target'), () => acMessage('room.result_target', { value1: data.branch, value2: (data.commit || '').slice(0, 12), value3: data.repo }));
    const metric = (label, value) => { const el = node('div', undefined, 'result-metric'); el.append(node('span', label, 'muted'), node('strong', String(value))); return el; };
    AC_I18N.replace($('result-metrics'), metric(() => acMessage('room.result_metric_files'), changed.length), metric(() => acMessage('room.result_metric_tests'), `${summary.passed}/${summary.total}`), metric(() => acMessage('room.result_metric_lessons'), notes.length));
    AC_I18N.replace($('result-files'), ...changed.map(f => row(f.path, `+${f.add} / −${f.del}`)));
    AC_I18N.replace($('result-tests'), ...outputs.map(t => row(t.task, () => acMessage(t.exit_code === 0 && !t.timed_out ? 'room.result_test_passed' : 'room.result_test_failed'))));
    AC_I18N.replace($('result-lessons'), ...(notes.length ? notes.map(l => node('li', l)) : [node('li', () => acMessage('room.result_no_lessons'), 'muted')]));
    // The dialog ships inside the Room section, which is hidden on the Workspace screen: hoist it so both can open it.
    if ($('result-dialog').parentElement !== document.body) document.body.append($('result-dialog'));
    if (!$('result-dialog').open) $('result-dialog').showModal();
  }
  return { files, tests, lessons, show };
})();
