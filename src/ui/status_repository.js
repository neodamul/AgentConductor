// GitHub #562: the footer's right side names the Workspace folder and its branch (mock `.status-repository`).
window.acStatusRepository = (() => {
  const NS = 'http://www.w3.org/2000/svg';
  const ICONS = {
    worktree: ['M3 7V5a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z', 'M3 8h18'],
    branch: ['M6 7v10M18 7a7 7 0 0 1-7 7H6'],
  };
  function icon(kind) {
    const svg = document.createElementNS(NS, 'svg');
    for (const [k, v] of Object.entries({ viewBox: '0 0 24 24', width: 13, height: 13, fill: 'none', stroke: 'currentColor', 'stroke-width': 1.6, 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'aria-hidden': 'true' })) svg.setAttribute(k, v);
    // GitHub #482: the circles are painted before the path, like the approved design (the overlap pixels differ otherwise).
    if (kind === 'branch') for (const [cx, cy] of [[6, 5], [6, 19], [18, 5]]) { const c = document.createElementNS(NS, 'circle'); c.setAttribute('cx', cx); c.setAttribute('cy', cy); c.setAttribute('r', 2); svg.append(c); }
    for (const d of ICONS[kind]) { const path = document.createElementNS(NS, 'path'); path.setAttribute('d', d); svg.append(path); }
    return svg;
  }
  function part(kind, text, title) {
    const box = document.createElement('span'); box.className = 'status-' + kind; box.title = title;
    const label = document.createElement('span'); label.textContent = text; box.append(icon(kind), label);
    return box;
  }
  // identity: { path, name, branch } or null (no folder chosen: nothing is shown).
  function render(footer, identity) {
    if (!footer) return;
    const stamp = JSON.stringify(identity || null) + document.documentElement.lang;
    let box = footer.querySelector('.status-repository');
    if (box && box.dataset.stamp === stamp) return;
    box?.remove();
    if (!identity) return;
    box = document.createElement('span'); box.className = 'status-repository'; box.dataset.stamp = stamp;
    box.setAttribute('aria-label', acMessage('startup.status_repository'));
    box.append(part('worktree', identity.name, acMessage('startup.status_worktree', { value1: identity.path })));
    if (identity.branch) box.append(part('branch', identity.branch, acMessage('startup.status_branch', { value1: identity.branch })));
    footer.append(box);
  }
  return { render };
})();
