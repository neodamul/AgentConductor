// Preserve authored descriptions while exposing clipped text on demand.
(() => {
  const generated = new WeakMap();
  document.addEventListener('pointerover', event => {
    for (let el = event.target instanceof Element ? event.target : event.target.parentElement; el; el = el.parentElement) {
      const previous = generated.get(el);
      if (previous && el.getAttribute('title') !== previous.generated) generated.delete(el);
      const record = generated.get(el);
      const style = getComputedStyle(el);
      const clipped = (style.textOverflow === 'ellipsis' || parseInt(style.webkitLineClamp, 10) > 0) &&
        (el.scrollWidth > el.clientWidth + 1 || el.scrollHeight > el.clientHeight + 1);
      if (!clipped) {
        if (record) { if (record.original === null) el.removeAttribute('title'); else el.setAttribute('title', record.original); generated.delete(el); }
        continue;
      }
      const text = el instanceof HTMLInputElement ? (el.type === 'password' ? '' : el.value.trim()) : el.textContent.trim(); if (!text) continue;
      const original = record ? record.original : el.getAttribute('title');
      const title = original && original !== text ? original + '\n' + text : text;
      el.setAttribute('title', title); generated.set(el, {original, generated:title});
    }
  });
})();
