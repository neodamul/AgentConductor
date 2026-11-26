(() => {
  const cache = new Map(), waiting = new Map(), current = new WeakMap();
  const encoder = new TextEncoder();
  let rendering = false;

  async function send(packet) {
    const handler = window.webkit?.messageHandlers?.agentconductor;
    const reply = handler
      ? await handler.postMessage(JSON.stringify(packet))
      : await window.__acRequest(JSON.stringify(packet));
    return JSON.parse(reply);
  }

  function paint(node, text, html) {
    const state = current.get(node);
    if (state?.text !== text || typeof html !== 'string') return;
    // HTML returned by the native Markdown renderer has escaped text and approved links only.
    node.innerHTML = html;
    node.dataset.rendered = '';
    state.afterPaint?.();
  }

  async function flush() {
    try {
      while (waiting.size) {
        const batch = [];
        let bytes = 0;
        for (const [text, entry] of waiting) {
          if (batch.length === 16 || bytes + entry.bytes > 56 * 1024) break;
          batch.push(text);
          bytes += entry.bytes;
        }
        let timer;
        const timeout = new Promise(resolve => { timer = setTimeout(() => resolve({}), 5000); });
        const result = await Promise.race([send({kind:'render_markdown', texts:batch}).catch(() => ({})), timeout]);
        clearTimeout(timer);
        const scroller = document.getElementById('activity')?.parentElement;
        const atBottom = scroller && scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < 70;
        for (const [index, text] of batch.entries()) {
          const html = result.html?.[index] ?? null;
          if (cache.size === 96) cache.delete(cache.keys().next().value);
          cache.set(text, html);
          for (const node of waiting.get(text).nodes) {
            if (node.isConnected) paint(node, text, html);
          }
          waiting.delete(text);
        }
        if (atBottom) scroller.scrollTop = scroller.scrollHeight;
      }
    } finally {
      rendering = false;
    }
  }

  function set(node, text = '', afterPaint) {
    if (current.get(node)?.text === text) return node;
    current.set(node, {text, afterPaint});
    node.textContent = text;
    node.classList.add('room-markdown');
    delete node.dataset.rendered;
    // Single-line plain text needs no native formatting. CSS handles the literal fallback.
    if (!text || !/[\n*_`\[~]/.test(text) && !/^\s*(?:#+\s|>\s?|[-+]\s|\d+\.\s)/.test(text)) return node;
    if (cache.has(text)) {
      paint(node, text, cache.get(text));
      return node;
    }
    const bytes = encoder.encode(JSON.stringify(text)).byteLength + 1;
    if (bytes > 56 * 1024) return node;
    if (!waiting.has(text)) waiting.set(text, {bytes, nodes:new Set()});
    const nodes = waiting.get(text).nodes;
    nodes.add(node);
    if (!rendering) {
      rendering = true;
      queueMicrotask(flush);
    }
    return node;
  }

  document.addEventListener('click', event => {
    const link = event.target.closest?.('.room-markdown a');
    if (!link) return;
    event.preventDefault();
    send({kind:'open_chat_link', url:link.getAttribute('href')}).catch(() => {});
  });
  window.acMarkdown = {set};
})();
