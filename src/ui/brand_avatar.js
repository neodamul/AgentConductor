// Clone only pinned, inert document assets; never interpret an icon URL or CLI-supplied markup.
window.acBrandAvatar = (provider, cls) => {
 if (!['claude', 'codex'].includes(provider)) return null;
 const source = document.getElementById('ac-brand-' + provider)?.content.firstElementChild;
 if (!source) return null;
 const svg = source.cloneNode(true);
 svg.removeAttribute('width'); svg.removeAttribute('height'); svg.setAttribute('class', 'brand-logo');
 // Like the mock (a box holding an inline svg): the box carries the border and padding, the svg fills its content.
 const box = document.createElement('span'); box.className = cls + ' app-icon'; box.setAttribute('aria-hidden', 'true'); box.append(svg);
 return box;
};
