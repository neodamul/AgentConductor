// GitHub #402 / DD-222: row actions of the Goal list — Result (the same centered popup as a completed run), the Goal
// folder in Finder/Explorer and the permanent deletion behind a centered confirmation (Cancel is the default).
// The requests are the startup requests goal_result, goal_folder and delete_goal; nothing is removed optimistically.
window.acGoalRows = (() => {
  const $ = id => document.getElementById(id);
  const SVG = 'http://www.w3.org/2000/svg';
  const FOLDER = 'M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z';
  const TRASH = 'M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3';
  function icon(path) {
    const svg = document.createElementNS(SVG, 'svg');
    for (const [k, v] of Object.entries({ viewBox: '0 0 24 24', width: 18, height: 18, fill: 'none', stroke: 'currentColor', 'stroke-width': 1.6, 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'aria-hidden': 'true' })) svg.setAttribute(k, v);
    const p = document.createElementNS(SVG, 'path'); p.setAttribute('d', path); svg.append(p); return svg;
  }
  function iconButton(cls, path, labelKey, slug, onclick) {
    const b = document.createElement('button'); b.type = 'button'; b.className = cls; b.append(icon(path));
    AC_I18N.bind(b, () => acMessage(labelKey, { value1: slug }), 'title'); AC_I18N.bind(b, () => acMessage(labelKey, { value1: slug }), 'aria-label');
    b.onclick = onclick; return b;
  }
  // Retries the same reviewed Goal and folder with a fresh revision while owned work drains; errors propagate.
  async function confirmDelete(request, fresh, target, { delay = 1000, tries = 60, sleep = ms => new Promise(r => setTimeout(r, ms)), onPending } = {}) {
    for (let i = 0; i < tries; i++) {
      const r = await request({ kind: 'delete_goal', revision: fresh(), slug: target.slug, goal: target.goal, root: target.root });
      if (r.deleted) return { deleted: true, changed: r.changed };
      onPending?.(i); await sleep(delay);
    }
    return { deleted: false, timedOut: true };
  }
  function review(ctx, s) {
    const dialog = $('start-delete-goal'), status = $('start-delete-goal-status'), confirm = $('start-delete-goal-confirm');
    ctx.request({ kind: 'goal_folder', slug: s.slug, open: false }).then(folder => {
      $('start-delete-goal-name').textContent = folder.goal; $('start-delete-goal-path').textContent = folder.root;
      status.textContent = ''; status.hidden = true; confirm.disabled = false; dialog.returnValue = '';
      confirm.onclick = async () => {
        confirm.disabled = true; status.hidden = false;
        try {
          const done = await confirmDelete(ctx.request, ctx.revision, { slug: folder.slug, goal: folder.goal, root: folder.root }, { onPending: () => AC_I18N.bind(status, () => acMessage('startup.delete_goal_pending')) });
          if (done.deleted) { dialog.close(); ctx.notice(() => acMessage('startup.delete_goal_done', { value1: s.slug })); await ctx.poll(); }
          else { status.textContent = acMessage('startup.delete_goal_timeout'); confirm.disabled = false; }
        } catch (e) { status.textContent = e.message; confirm.disabled = false; }
      };
      dialog.showModal();
    }).catch(e => ctx.notice(e.message, true));
  }
  async function showResult(ctx, s) {
    const r = await ctx.request({ kind: 'goal_result', slug: s.slug });
    if (!r.report) { ctx.notice(() => acMessage('startup.goal_row_no_result'), true); return; }
    const phaseResults = r.phase_results?.length > 1 ? r.phase_results : null;
    acResultView.show({ summary: r.report.latest_review?.summary, goal: r.goal || r.report.plan?.goal || s.slug, branch: r.report.branch, repo: r.report.repo, commit: r.evidence?.report?.candidate?.commit, diff: r.evidence?.diff, tests: r.evidence?.test_output, details: r.details, phaseResults });
  }
  // item: the row built by startup.js; ctx: { request, revision, poll, notice, busy, actions: [existing row buttons] }.
  function decorate(item, s, ctx) {
    const info = document.createElement('div'); info.className = 'goal-row-info'; info.append(...item.childNodes);
    // GitHub #753: the goal text keeps its line breaks, shows two lines and expands on demand.
    const text = info.querySelector('.small.muted');
    if (text && text.textContent.trim()) {
      const toggle = document.createElement('button'); toggle.type = 'button'; toggle.className = 'goal-row-toggle'; toggle.setAttribute('aria-expanded', 'false');
      const label = () => acMessage(toggle.getAttribute('aria-expanded') === 'true' ? 'startup.goal_row_collapse' : 'startup.goal_row_expand');
      text.classList.add('goal-row-text'); AC_I18N.bind(toggle, label);
      toggle.onclick = () => { const open = toggle.getAttribute('aria-expanded') !== 'true'; toggle.setAttribute('aria-expanded', String(open)); text.classList.toggle('open', open); AC_I18N.bind(toggle, label); };
      text.after(toggle);
    }
    const actions = document.createElement('div'); actions.className = 'goal-row-actions';
    const folder = iconButton('goal-folder', FOLDER, 'startup.goal_row_folder', s.slug, () => ctx.request({ kind: 'goal_folder', slug: s.slug, open: true }).catch(e => ctx.notice(e.message, true)));
    actions.append(folder);
    if (s.progress?.stage === 'completed') {
      const result = document.createElement('button'); result.type = 'button'; result.className = 'quiet small-button goal-result';
      AC_I18N.bind(result, () => acMessage('startup.goal_row_result')); result.onclick = () => showResult(ctx, s).catch(e => ctx.notice(e.message, true)); actions.append(result);
    }
    actions.append(...ctx.actions);
    const trash = iconButton('goal-trash', TRASH, 'startup.goal_row_trash', s.slug, () => review(ctx, s));
    trash.disabled = ctx.busy(); folder.disabled = ctx.busy();
    item.classList.add('goal-list-row'); item.replaceChildren(trash, info, actions);
  }
  return { confirmDelete, decorate };
})();
