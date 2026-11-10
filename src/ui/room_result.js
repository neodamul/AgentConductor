// GitHub #398 / DD-222: the Room opens the centered result popup (result_view.js) once when a watched run completes
// and from the Result button; it reads the existing evidence and task requests.
// Opens once when a watched run completes; a run that was already finished when the Room opened does not pop up.
function resultTransition(prev, next) { return next === 'completed' && !!prev && prev !== 'completed'; }
let resultState;
function watchResult(snap) {
  const state = snap.report.state;
  $('show-result').hidden = state !== 'completed';
  if (resultTransition(resultState, state)) openResult().catch(e => notice(e.message, true));
  resultState = state;
}
async function openResult() {
  const r = snapshot.report, review = await request({ kind: 'evidence' });
  const details = await Promise.all(r.tasks.map((_, index) => request({ kind: 'task', index }).catch(() => ({}))));
  acResultView.show({ summary: r.latest_review?.summary, goal: r.plan.goal, branch: r.branch, repo: r.repo, commit: review.report.candidate?.commit, diff: review.diff, tests: review.test_output, details });
}
