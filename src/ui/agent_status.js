// GitHub #410: an agent card's avatar border carries the agent's task state (mock `.status-icon[data-state]`):
// green running/completed, yellow waiting, red failure; a small marker flags a decision or error.
// The Room snapshot feeds `apply`; the cards (rebuilt by startup.js) are re-stamped when they change.
window.acAgentStatus = (() => {
  const TONE = { bad: 'error', wait: 'waiting', run: 'running' };
  let states = {};
  function stamp() {
    for (const card of document.querySelectorAll('#start-profiles .startup-profile, #rail-pane-agents .startup-profile')) {
      const symbol = card.querySelector('.agent-symbol'), id = card.dataset.agent;
      if (!symbol) continue;
      const state = states[id];
      if (state) symbol.dataset.state = state; else delete symbol.dataset.state;
      if (state === 'error' || state === 'waiting') symbol.dataset.attention = 'true'; else delete symbol.dataset.attention;
    }
  }
  // tasks: [{status}], plan: [{agent}] in the same order; the strongest state of an agent's tasks wins.
  function apply(tasks, plan, tone) {
    const next = {};
    tasks.forEach((task, i) => {
      const id = plan[i]?.agent, t = tone(task.status);
      if (!id) return;
      const state = TONE[t] || (t === 'ok' ? 'completed' : null);
      const rank = s => ['completed', 'running', 'waiting', 'error'].indexOf(s);
      if (state && (!next[id] || rank(state) > rank(next[id]))) next[id] = state;
      else if (!state && next[id] === 'completed') delete next[id]; // an idle task keeps the agent from reading as done
    });
    states = next; stamp();
  }
  const host = document.getElementById('start-profiles');
  if (host) new MutationObserver(stamp).observe(host, { childList: true });
  return { apply, stamp };
})();
