// GitHub #751: the application menu groups related actions into two levels (group, then item); the Workspace group holds everything about the Workspace;
// preferences and maintenance actions live in Settings. A group opens by click, hover, ArrowRight/Enter/Space; Escape closes the submenu first.
window.acAppMenu = (() => {
  const click = id => document.getElementById('start-' + id)?.click();
  function closeGroups(menu, except) {
    menu.querySelectorAll('.menu-parent').forEach(parent => { const sub = parent.nextElementSibling, open = sub === except; sub.hidden = !open; parent.setAttribute('aria-expanded', String(open)); });
  }
  function leaf(el, key, run) { const b = el('button', () => acMessage(key)); b.setAttribute('role', 'menuitem'); b.onclick = run; return b; }
  function group(el, key, items) {
    const wrap = el('div', undefined, 'menu-group'), parent = el('button', () => acMessage(key), 'menu-parent'), sub = el('div', undefined, 'submenu');
    parent.setAttribute('aria-haspopup', 'menu'); parent.setAttribute('aria-expanded', 'false'); sub.setAttribute('role', 'menu'); sub.hidden = true; sub.append(...items); wrap.append(parent, sub);
    const open = focus => { closeGroups(wrap.parentElement, sub); if (focus) sub.querySelector('button').focus(); };
    parent.onclick = () => (sub.hidden ? open(false) : closeGroups(wrap.parentElement, null));
    wrap.onmouseenter = () => open(false);
    parent.onkeydown = e => { if (['ArrowRight', 'Enter', ' '].includes(e.key)) { e.preventDefault(); open(true); } };
    sub.onkeydown = e => { if (e.key === 'ArrowLeft' || e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); closeGroups(wrap.parentElement, null); parent.focus(); } };
    return wrap;
  }
  function bindArrows(list) {
    if (list.dataset.arrows) return; list.dataset.arrows = '1';
    list.addEventListener('keydown', e => {
      if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return; e.preventDefault();
      const sub = document.activeElement.closest('.submenu'), pool = sub ? [...sub.querySelectorAll(':scope>button,:scope>.workspace-row>button')] : [...list.querySelectorAll(':scope>button,:scope>.menu-group>.menu-parent')];
      if (pool.length) pool[(pool.indexOf(document.activeElement) + (e.key === 'ArrowDown' ? 1 : -1) + pool.length) % pool.length].focus();
    });
  }
  // create/open/updates/about are the existing Workspace and update buttons; close hides the menu; the rest reuse the titlebar and Settings controls.
  function items({ el, rows, create, open, updates, about, close, rules, messenger, exportWorkspace, importWorkspace, tools, manual, report, releaseNotes }) {
    const act = (key, run) => leaf(el, key, () => { close(); run(); });
    const nodes = [
      group(el, 'startup.menu_workspace', [...rows, ...(rows.length ? [el('hr')] : []), create, open, act('startup.export_settings', exportWorkspace), act('startup.import_settings', importWorkspace)]),
      // Plan rules and Messenger sit one level up, beside the groups; the Goal list and New Goal live on the screen itself.
      act('startup.plan_rules', rules),
      act('startup.messenger', messenger),
      act('startup.menu_settings', () => click('settings-button')),
      group(el, 'startup.menu_help', [updates, act('startup.release_notes', releaseNotes), act('startup.report_bug', report), act('startup.user_manual', manual), act('startup.agent_tools_guide', tools), about]),
      el('hr'),
      act('startup.menu_close_window', () => window.__acCloseRequested?.()),
    ];
    queueMicrotask(() => bindArrows(document.getElementById('start-workspace-list')));
    return nodes;
  }
  return { items };
})();
