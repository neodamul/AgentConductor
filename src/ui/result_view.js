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
      if (header) { current = { path: header[1], add: 0, del: 0, lines: [] }; list.push(current); continue; }
      if (!current) continue;
      // GitHub #826: keep the hunks so a row can unfold its diff; bounded so a huge file cannot flood the popup.
      if (/^(index |--- |\+\+\+ |new file|deleted file|similarity|rename )/.test(line)) continue;
      if (current.lines.length < 400) current.lines.push(line);
      if (line.startsWith('+')) current.add++;
      else if (line.startsWith('-')) current.del++;
    }
    return list;
  }
  function tests(outputs) { const list = outputs || []; return { passed: list.filter(t => t.exit_code === 0 && !t.timed_out).length, total: list.length }; }
  function phaseFiles(phases) { const map = new Map(); for (const phase of phases || []) for (const file of files(phase.evidence?.diff)) { const item = map.get(file.path) || { path: file.path, phases: [], lines: [] }; item.phases.push(phase.phase); item.lines.push(...file.lines); map.set(file.path, item); } return [...map.values()]; }
  function phaseTests(phases) { return (phases || []).flatMap(phase => (phase.evidence?.test_output || []).map(test => ({ ...test, task: `${phase.phase} · ${test.task}` }))); }
  function lessons(details) {
    const out = [];
    for (const d of details || []) { for (const l of d.step_lessons || []) out.push(l.response); for (const k of d.knowledge || []) out.push(k.lesson?.response); }
    return [...new Set(out.filter(Boolean))];
  }
  function syncSection(section) {
    const summary = section.querySelector('summary'), toggle = summary.querySelector('.result-section-toggle');
    summary.setAttribute('aria-expanded', String(section.open));
    AC_I18N.bind(toggle, () => acMessage(section.open ? 'room.result_collapse' : 'room.result_expand'));
  }
  // data: { goal, branch, repo, commit, diff, tests (test_output list), details (per-task data) }
  function show(data) {
    const changed = data.phaseResults?.length ? phaseFiles(data.phaseResults) : files(data.diff), outputs = data.phaseResults?.length ? phaseTests(data.phaseResults) : data.tests || [], summary = tests(outputs), notes = lessons(data.details);
    const row = (a, b) => { const el = node('li'); el.append(node('span', a), node('span', () => ' · ' + (typeof b === 'function' ? b() : b))); return el; };
    // GitHub #733/#737: the title is the Goal's first meaningful line, then the coordinator's summary sentence.
    const goalLine = String(data.goal || '').split('\n').map(l => l.trim()).find(l => l && !/^<\/?[\w-]+>$/.test(l)) || '';
    // GitHub #812: a Phase with Phases left reads as that Phase's result and continues to the next Phase's plan.
    const next = data.nextPhase;
    $('result-dialog').dataset.nextPhase = next ? next.id : '';
    AC_I18N.bind($('result-eyebrow'), () => next ? acMessage('room.result_eyebrow_phase', { value1: next.number - 1, value2: next.total }) : acMessage('room.result_eyebrow'));
    $('result-close-icon').textContent = next ? '→ ' : '☰ ';
    AC_I18N.bind($('result-close-label'), () => next ? acMessage('room.result_next_phase', { value1: next.number, value2: next.total }) : acMessage('room.result_to_goals'));
    AC_I18N.bind($('result-title'), () => next ? acMessage('room.result_title_phase', { value1: goalLine, value2: next.number - 1 }) : acMessage('room.result_title', { value1: goalLine }));
    $('result-title').title = goalLine;
    $('result-summary').textContent = data.phaseResults?.length ? acMessage('room.result_all_phases', { value1: data.phaseResults.length, value2: data.summary || '' }) : data.summary || ''; $('result-summary').hidden = !data.phaseResults?.length && !data.summary;
    AC_I18N.bind($('result-target'), () => acMessage('room.result_target', { value1: data.branch, value2: (data.commit || '').slice(0, 12) }));
    AC_I18N.bind($('result-location'), () => acMessage('room.result_location', { value1: data.repo }));
    const metric = (label, value) => { const el = node('div', undefined, 'result-metric'); el.append(node('span', label), node('strong', String(value))); return el; };
    AC_I18N.replace($('result-metrics'), metric(() => acMessage('room.result_metric_files'), changed.length), metric(() => acMessage('room.result_metric_tests'), `${summary.passed}/${summary.total}`), metric(() => acMessage('room.result_metric_lessons'), notes.length));
    // GitHub #826: each changed file is a button that unfolds its diff below it.
    const fileRow = f => {
      const item = node('li', undefined, 'file-row'), button = node('button', undefined, 'file-diff-toggle'), pre = node('pre', undefined, 'file-diff');
      button.type = 'button'; button.setAttribute('aria-expanded', 'false'); pre.hidden = true;
      button.append(node('span', f.path), node('span', () => ' · ' + (f.phases ? f.phases.join(' · ') : `+${f.add} / −${f.del}`)));
      for (const text of f.lines || []) pre.append(node('span', text || ' ', 'diff-line' + (text.startsWith('@@') ? ' hunk' : text.startsWith('+') ? ' add' : text.startsWith('-') ? ' remove' : '')));
      button.disabled = !(f.lines || []).length;
      button.onclick = () => { const open = pre.hidden; pre.hidden = !open; button.setAttribute('aria-expanded', String(open)); };
      item.append(button, pre); return item;
    };
    AC_I18N.replace($('result-files'), ...changed.map(fileRow));
    AC_I18N.replace($('result-tests'), ...outputs.map(t => row(t.task, () => acMessage(t.exit_code === 0 && !t.timed_out ? 'room.result_test_passed' : 'room.result_test_failed'))));
    AC_I18N.replace($('result-lessons'), ...(notes.length ? notes.map(l => node('li', l)) : [node('li', () => acMessage('room.result_no_lessons'), 'muted')]));
    // GitHub #821: the popup owns its buttons, so they work from the Goal list before the Room script has run.
    $('result-x').onclick = () => $('result-dialog').close();
    $('result-close').onclick = () => {
      const next = $('result-dialog').dataset.nextPhase; $('result-dialog').close();
      if (next) window.acNextPhase?.plan();
      else if (!document.getElementById('start-goals')?.open) document.getElementById('start-goal-list-open')?.click();
    };
    const sections = [...$('result-dialog').querySelectorAll('details.result-section')];
    sections.forEach((section, index) => {
      section.querySelector('.result-section-count').textContent = ` · ${[changed.length, outputs.length, notes.length][index]}`;
      if (!section.dataset.toggleBound) { section.addEventListener('toggle', () => syncSection(section)); section.dataset.toggleBound = 'true'; }
    });
    // The dialog ships inside the Room section, which is hidden on the Workspace screen: hoist it so both can open it.
    if ($('result-dialog').parentElement !== document.body) document.body.append($('result-dialog'));
    // GitHub #799: each opening starts with the sections collapsed.
    if (!$('result-dialog').open) sections.forEach(section => { section.open = false; });
    sections.forEach(syncSection);
    if (!$('result-dialog').open) $('result-dialog').showModal();
  }
  return { files, tests, lessons, phaseFiles, phaseTests, show };
})();
