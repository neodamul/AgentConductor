// Clone only pinned, inert document assets; never interpret an icon URL or CLI-supplied markup.
window.acBrandAvatar = (provider, cls) => {
 if (!['claude', 'codex', 'opencode'].includes(provider)) return null;
 const source = document.getElementById('ac-brand-' + provider)?.content.firstElementChild;
 if (!source) return null;
 const svg = source.cloneNode(true);
 // GitHub #697: gradient ids must be unique per copy; when the copy that owned the id leaves the page (a re-rendered row), the others lose their fill.
 acBrandAvatar.count = (acBrandAvatar.count || 0) + 1;
 for (const owner of svg.querySelectorAll('[id]')) {
  const old = owner.id, next = old + '-' + acBrandAvatar.count;
  owner.id = next;
  for (const user of svg.querySelectorAll('*')) for (const name of ['fill', 'stroke', 'style', 'href', 'xlink:href']) {
   const value = user.getAttribute(name);
   if (value && value.includes('#' + old)) user.setAttribute(name, value.split('#' + old + ')').join('#' + next + ')').split('#' + old + '"').join('#' + next + '"'));
  }
 }
 svg.removeAttribute('width'); svg.removeAttribute('height'); svg.setAttribute('class', 'brand-logo');
 // Like the mock (a box holding an inline svg): the box carries the border and padding, the svg fills its content.
 const box = document.createElement('span'); box.className = cls + ' app-icon'; box.setAttribute('aria-hidden', 'true'); box.append(svg);
 return box;
};
