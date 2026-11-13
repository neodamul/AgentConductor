// GitHub #686: the plan card's heading line, the ownership summary under the task rows and the detail strip of the selected task (mock step 3).
const acPlanReview = (() => {
  const overlap = (a, b) => a.some(x => b.some(y => x === y || x.startsWith(y.endsWith('/') ? y : y + '/') || y.startsWith(x.endsWith('/') ? x : x + '/')));
  const conflicts = tasks => tasks.flatMap((t, i) => tasks.slice(i + 1).filter(o => overlap(t.paths || [], o.paths || [])).map(o => `${t.id} / ${o.id}`));
  function detail(task, title) {
    const paths = (task.paths || []).join(', ') || '—', deps = (task.depends_on || []).join(', ') || null, test = (task.test_command || []).join(' ') || '—';
    return acMessage('startup.plan_detail', { value1: task.id, value2: title, value3: paths, value4: deps || acMessage('startup.plan_no_dependencies'), value5: test });
  }
  return {
    meaningfulBrief(spec) {
      const ids = new Set((spec.specs || []).map(task => task.id));
      const lines = String(spec.brief || "").split(/\r?\n/).map(line => line.trim()).filter(Boolean);
      return lines.some(line => !/^Tasks?:$/i.test(line) && !ids.has(line.replace(/^[-*]\s+/, "")));
    },
    // `box` holds the two lines; `select(index)` re-renders the strip when another row is opened.
    render(box, tasks, selected = 0, titleOf = task => task.id, phase = {phase: 1, phase_total: 1}) {
      box.replaceChildren();
      if (!tasks.length) { box.hidden = true; return; }
      box.hidden = false;
      const picked = tasks[Math.min(selected, tasks.length - 1)], title = titleOf(picked) || picked.id, clash = conflicts(tasks);
      const line = document.createElement('p'), strip = document.createElement('p');
      line.className = 'plan-position'; strip.className = 'plan-detail';
      AC_I18N.bind(line, () => acMessage('startup.plan_position', { ...phase, value1: Math.min(selected, tasks.length - 1) + 1, value2: tasks.length, value3: title, value4: clash.length ? acMessage('startup.plan_paths_overlap', { value1: clash.join(', ') }) : acMessage('startup.plan_paths_clear') }));
      AC_I18N.bind(strip, () => detail(picked, title));
      box.append(line, strip);
    },
  };
})();
