// GitHub #706: the integration verification card is the mock's step 5: the test rows and the coordinator's judgments, a footer with a note,
// Stop and a disabled "wait for verification" button; no tabs, no direction field, no status pill inside the card.
function isVerifying(r) { return r.state === 'running' && r.tasks.length > 0 && r.tasks.every(t => t.status === 'completed'); }
function renderVerifyFooter(r) {
  const card = document.querySelector('#run-screen .room-card'), footer = $('verify-footer');
  const active = coordinatorView && isVerifying(r);
  card.classList.toggle('coordinator-card', !!coordinatorView);
  card.classList.toggle('verifying', active);
  footer.hidden = !active;
  $('verify-stop').disabled = $('stop').disabled;
}
function wireVerifyFooter() { $('verify-stop').onclick = () => $('stop').click(); }
