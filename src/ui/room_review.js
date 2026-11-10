// GitHub #689: the approval-waiting card is the mock's: the summary, then a footer with Reject, Request changes and Review changes
// (Approve once the evidence has been opened and the candidate can be approved). The tabs and the direction field wait behind them.
let reviewOpen = false, reviseOpen = false, reviewState = '';
function renderReviewFooter(r) {
  const card = document.querySelector('#run-screen .room-card'), footer = $('review-footer');
  const active = coordinatorView && r.state === 'waiting_approval';
  if (r.state !== reviewState) { reviewState = r.state; reviewOpen = false; reviseOpen = false; }
  card.classList.toggle('review-only', active && !reviewOpen && !reviseOpen);
  card.classList.toggle('revising', active && reviseOpen && !reviewOpen);
  footer.hidden = !active;
  if (!active) return;
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
    reviewOpen = true; changeTab('review'); renderReviewFooter(snapshot.report); loadReview();
  };
}
