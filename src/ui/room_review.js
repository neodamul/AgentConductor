// GitHub #689: the approval-waiting card is the mock's: the summary, then a footer with Reject, Request changes and Review changes
// (Approve once the evidence has been opened and the candidate can be approved). The tabs and the direction field wait behind them.
let reviewOpen = false, reviseOpen = false, reviewState = '';
function renderReviewFooter(r) {
  const card = document.querySelector('#run-screen .room-card'), footer = $('review-footer');
  // The approved state keeps the footer: its primary button applies the approved changes (the old status box's action).
  const approved = coordinatorView && r.state === 'approved', done = coordinatorView && r.state === 'completed', active = coordinatorView && r.state === 'waiting_approval' || approved || done;
  if (r.state !== reviewState) { reviewState = r.state; reviewOpen = false; reviseOpen = false; }
  card.classList.toggle('review-only', active && !reviewOpen && !reviseOpen);
  card.classList.toggle('revising', active && reviseOpen && !reviewOpen);
  footer.hidden = !active;
  if (!active) return;
  $('review-deny').hidden = approved || done; $('review-revise').hidden = approved;
  // GitHub #728: a finished run is the mock's: Request changes and View the finished result, in place of the title-bar button.
  if (done) { $('review-primary').dataset.mode = 'result'; AC_I18N.bind($('review-primary'), () => acMessage('room.result_view_done')); $('review-primary').disabled = $('show-result').hidden; return; }
  if (approved) { $('review-primary').dataset.mode = 'merge'; AC_I18N.bind($('review-primary'), () => $('merge').textContent); $('review-primary').disabled = $('merge').disabled; return; }
  $('review-primary').disabled = false;
  const approvable = !$('approve').disabled;
  $('review-primary').dataset.mode = approvable ? 'approve' : 'open';
  AC_I18N.bind($('review-primary'), () => acMessage(approvable ? 'room.approve' : 'room.review_open_changes'));
  $('review-deny').disabled = $('reject').disabled;
}
function wireReviewFooter() {
  $('review-deny').onclick = () => $('reject').click();
  $('review-revise').onclick = () => { reviseOpen = true; renderReviewFooter(snapshot.report); $('feedback').focus(); };
  $('review-primary').onclick = () => {
    if ($('review-primary').dataset.mode === 'approve') { $('approve').click(); return; }
    if ($('review-primary').dataset.mode === 'merge') { $('merge').click(); return; }
    if ($('review-primary').dataset.mode === 'result') { $('show-result').click(); return; }
    reviewOpen = true; changeTab('review'); renderReviewFooter(snapshot.report); loadReview();
  };
}
