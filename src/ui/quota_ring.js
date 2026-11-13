// GitHub #426: the dual-ring quota icon beside an agent name and its popup (mock `.usage-button`, `.usage-popover`).
// Data is the backend's `quota_indicators[agent]`: rings (0-2 rows) draw the icon, windows and budgets fill the popup.
// Rendering never queries a provider; refresh uses the existing quota request (`options.refresh`).
window.acQuotaRing = (() => {
  const NS = 'http://www.w3.org/2000/svg', RINGS = [{ r: 11, w: 2.3, tone: 'quota-short' }, { r: 6.5, w: 2.1, tone: 'quota-long' }];
  const entries = new Map();
  let popup = null, openId = null, opener = null;
  const svg = (tag, attrs) => { const e = document.createElementNS(NS, tag); for (const k in attrs) e.setAttribute(k, attrs[k]); return e; };
  const text = (tag, value, cls) => { const e = document.createElement(tag); if (cls) e.className = cls; e.textContent = value; return e; };
  const pct = row => row && Number.isFinite(row.remaining_percent) ? Math.max(0, Math.min(100, row.remaining_percent)) : null;
  // GitHub #679: a workspace without a project folder is a state to explain, not a failed lookup.
  const failed = indicator => !!(indicator && indicator.error && indicator.error_code !== 'no_project_folder');
  const same = (a, b) => a && b && a.kind === b.kind && a.index === b.index;

  function icon(indicator) {
    const s = svg('svg', { viewBox: '0 0 30 30', 'aria-hidden': 'true' });
    RINGS.forEach(({ r, w, tone }, i) => {
      s.append(svg('circle', { class: 'usage-track', cx: 15, cy: 15, r, 'stroke-width': w }));
      const value = failed(indicator) ? null : pct(indicator?.rings?.[i]);
      if (value !== null) s.append(svg('circle', { class: 'usage-value ' + tone, cx: 15, cy: 15, r, 'stroke-width': w, pathLength: 100, 'stroke-dasharray': value + ' 100', transform: 'rotate(-90 15 15)' }));
    });
    if (failed(indicator)) s.append(Object.assign(svg('text', { x: 15, y: 19, 'text-anchor': 'middle', fill: 'currentColor', 'font-size': 13, 'font-weight': 700 }), { textContent: '!' }));
    return s;
  }

  function row(item, tone) {
    const value = pct(item), box = text('div', '', 'usage-item' + (tone ? ' ' + tone : '')), line = text('div', '', 'usage-line');
    line.append(text('span', item.label || ''), text('b', item.unlimited ? acMessage('startup.quota_unlimited') : value === null ? '—' : Math.round(value) + '%'));
    const meter = text('div', '', 'usage-meter'), fill = document.createElement('span'); fill.style.width = (item.unlimited ? 100 : value ?? 0) + '%'; meter.append(fill);
    box.append(line, meter);
    const detail = item.kind === 'budget' && item.allocated != null ? [(item.remaining ?? '-') + ' / ' + item.allocated + (item.unit ? ' ' + item.unit : ''), item.reset_label].filter(Boolean).join(' · ') : item.reset_label;
    if (detail) box.append(text('div', detail, 'usage-reset'));
    return box;
  }

  function observed(indicator) {
    const at = Number(indicator?.observed_at);
    if (!at) return acMessage('startup.quota_not_observed');
    const age = Math.max(0, Date.now() / 1000 - at);
    const when = age < 90 ? acMessage('startup.quota_observed_now') : acMessage('startup.quota_observed_at', { value1: new Date(at * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) });
    return indicator.source ? when + ' · ' + indicator.source : when;
  }

  function fill(id) {
    const entry = entries.get(id), indicator = entry?.indicator;
    if (!popup || !entry) return;
    const error = failed(indicator), body = popup.querySelector('.usage-body');
    popup.querySelector('.usage-agent').textContent = entry.name || id;
    popup.querySelector('.usage-heading>.usage-reset').textContent = error ? acMessage('startup.quota_error_label') : acMessage('startup.quota_remaining');
    body.replaceChildren();
    if (error) {
      const box = text('div', '', 'usage-error'); box.setAttribute('role', 'alert');
      box.append(text('strong', acMessage('startup.quota_error_title')), text('p', acMessage('startup.quota_error_message')), text('p', String(indicator.error)));
      body.append(box);
    } else if (indicator?.error_code === 'no_project_folder') {
      body.append(text('p', acMessage('startup.quota_needs_folder'), 'usage-unavailable'));
    } else if (indicator?.status === 'unsupported' || !indicator) {
      body.append(text('p', acMessage('startup.quota_unsupported'), 'usage-unavailable'));
    } else {
      const rings = indicator.rings || [];
      for (const item of [...(indicator.windows || []), ...(indicator.budgets || [])]) { const at = rings.findIndex(r => same(r, item)); body.append(row(item, at >= 0 ? RINGS[at].tone : '')); }
      if (indicator.status === 'stale') body.append(text('p', acMessage('startup.quota_stale'), 'usage-unavailable'));
    }
    popup.querySelector('.usage-observed>span').textContent = observed(indicator);
    const refresh = popup.querySelector('.usage-refresh');
    // #933: icon only, as in the mock; the label tells retry from refresh.
    const label = acMessage(error ? 'startup.quota_retry' : 'startup.quota_refresh'); refresh.setAttribute('aria-label', label); refresh.title = label;
    refresh.hidden = !entry.canRefresh; refresh.disabled = !!entry.busy; refresh.classList.toggle('refreshing', !!entry.busy);
  }

  function close(focus) { if (!popup) return; popup.hidden = true; opener?.setAttribute('aria-expanded', 'false'); const back = opener; openId = opener = null; if (focus) back?.focus(); }

  function place(button) {
    const rect = button.getBoundingClientRect();
    popup.style.left = Math.round(Math.min(innerWidth - popup.offsetWidth - 12, Math.max(12, rect.right + 10))) + 'px';
    popup.style.top = Math.round(Math.min(innerHeight - popup.offsetHeight - 12, Math.max(12, rect.top))) + 'px';
  }

  function build() {
    popup = text('div', '', 'usage-popover'); popup.id = 'usage-popover'; popup.setAttribute('role', 'dialog'); popup.setAttribute('aria-label', acMessage('startup.quota_dialog')); popup.hidden = true;
    const heading = text('div', '', 'usage-heading'), closeButton = text('button', '×', 'usage-close'); closeButton.type = 'button';
    heading.append(text('strong', '', 'usage-agent'), text('span', '', 'sp'), text('span', '', 'usage-reset'), closeButton);
    const foot = text('div', '', 'usage-observed'), refresh = text('button', '', 'usage-refresh'); refresh.type = 'button'; refresh.append(text('span', '↻', 'usage-glyph'));
    foot.append(document.createElement('span'), refresh);
    popup.append(heading, text('div', '', 'usage-body'), foot);
    closeButton.onclick = () => close(true);
    refresh.onclick = () => { const entry = entries.get(openId); if (entry && !entry.busy) entry.refresh?.(); };
    document.addEventListener('keydown', e => { if (e.key === 'Escape' && openId) close(true); });
    document.addEventListener('click', e => { if (openId && !popup.contains(e.target) && !opener?.contains(e.target)) close(false); });
    document.body.append(popup);
  }

  // options: { busy, canRefresh, refresh } -- returns the ring button for the agent card.
  function button(id, indicator, options = {}) {
    entries.set(id, { indicator, name: options.name, busy: !!options.busy, canRefresh: !!options.canRefresh, refresh: options.refresh });
    const b = document.createElement('button'); b.type = 'button'; b.className = 'usage-button' + (failed(indicator) ? ' error' : '');
    b.setAttribute('aria-controls', 'usage-popover'); b.setAttribute('aria-expanded', String(openId === id));
    const summary = failed(indicator) ? acMessage('startup.quota_open_error') : (indicator?.rings || []).map(r => r.label + ' ' + (pct(r) === null ? '—' : Math.round(pct(r)) + '%')).join(' · ');
    b.title = summary ? (failed(indicator) ? summary : summary + ' ' + acMessage('startup.quota_left')) : acMessage('startup.quota_none');
    b.setAttribute('aria-label', acMessage('startup.quota_open', { value1: id }));
    b.append(icon(indicator));
    b.onclick = () => { if (openId === id) { close(false); return; } if (!popup) build(); close(false); openId = id; opener = b; fill(id); popup.hidden = false; b.setAttribute('aria-expanded', 'true'); place(b); };
    if (openId === id) { opener = b; fill(id); }
    return b;
  }
  return { button, close };
})();
