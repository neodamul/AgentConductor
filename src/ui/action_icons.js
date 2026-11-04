// GitHub #408 / DD-222: step-card action buttons carry a leading 13px icon that matches the action. The mapping is by
// button id (labels are translated); the icon itself is a CSS mask on [data-action-icon], so no markup text changes.
(() => {
  const ICONS = { 'start-save': 'save', 'start-prepare': 'refresh', 'start-run': 'play', 'start-read-plan': 'list', 'start-draft': 'edit', 'start-discard': 'undo', 'send-feedback': 'send', stop: 'stop' };
  for (const [id, icon] of Object.entries(ICONS)) { const button = document.getElementById(id); if (button) button.dataset.actionIcon = icon; }
})();
