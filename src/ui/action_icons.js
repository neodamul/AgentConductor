// GitHub #408 / DD-222: step-card action buttons carry a leading 13px icon that matches the action. The mapping is by
// button id (labels are translated); the icon itself is a CSS mask on [data-action-icon], so no markup text changes.
(() => {
  const ICONS = { 'start-save': 'save', 'start-prepare': 'refresh', 'start-run': 'check', 'start-regenerate': 'refresh', 'start-roles-suggest': 'plus', 'start-roles-next': 'save', 'start-optimize-button': 'sliders', 'start-read-plan': 'list', 'start-draft': 'edit', 'start-discard': 'back', 'start-browse': 'folder', 'start-new-goal': 'plus', 'start-jira-reference': 'link', 'start-confluence-reference': 'link', 'send-feedback': 'send', stop: 'stop', 'verify-stop': 'stop', 'verify-wait': 'next', 'review-revise': 'next', 'denied-reassign': 'next', 'review-deny': 'reject', 'tool-decision-deny': 'reject', 'tool-decision-allow': 'check', 'denied-resume': 'play', 'review-primary': 'check' };
  for (const [id, icon] of Object.entries(ICONS)) { const button = document.getElementById(id); if (button) button.dataset.actionIcon = icon; }
})();
