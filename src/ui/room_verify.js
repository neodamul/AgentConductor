// GitHub #706: the integration verification card is the mock's step 5: the test rows and the coordinator's judgments, a footer with a note,
// Stop and a disabled "wait for verification" button; no tabs, no direction field, no status pill inside the card.
function isVerifying(r) { return r.state === 'running' && r.tasks.length > 0 && r.tasks.every(t => t.status === 'completed'); }
function renderVerifyFooter(r) {
  const card = document.querySelector('#run-screen .room-card'), footer = $('verify-footer');
  const active = coordinatorView && isVerifying(r);
  card.classList.toggle('coordinator-card', !!coordinatorView);
  card.classList.toggle('verifying', active);
  footer.hidden = !active;
  $('verify-judgments').hidden = !active;
  $('verify-stop').disabled = $('stop').disabled;
}
function wireVerifyFooter() { $('verify-stop').onclick = () => $('stop').click(); }
// GitHub #727: a task the CLI refused a tool is the mock's denied card: a note, hand the task to another agent, or resume; no direction field.
function renderDeniedFooter(waiting) {
  const t = task(), on = !coordinatorView && !waiting && t?.status === 'permission_denied';
  $('denied-footer').hidden = !on;
  $('denied-resume').disabled = $('resume').disabled;
  return on;
}
function wireDeniedFooter() {
  $('denied-resume').onclick = () => $('resume').click();
  $('denied-reassign').onclick = () => { inspectorState.open = true; paintInspector(); const s = $('agent-select'); (window.acChoices && acChoices.buttonFor(s) || s).focus(); };
}
