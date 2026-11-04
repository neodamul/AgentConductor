// Clone only pinned, inert document assets; never interpret an icon URL or CLI-supplied markup.
window.acBrandAvatar = (provider, cls) => {
 if (!['claude', 'codex'].includes(provider)) return null;
 const source = document.getElementById('ac-brand-' + provider)?.content.firstElementChild;
 if (!source) return null;
 const svg = source.cloneNode(true);
 svg.removeAttribute('width'); svg.removeAttribute('height');
 svg.setAttribute('class', cls + ' app-icon'); svg.setAttribute('aria-hidden', 'true');
 return svg;
};
