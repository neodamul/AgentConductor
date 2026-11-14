// #927: errors belong to the task card; recovery uses its footer, not a second run banner.
let repairOpen=false,recoveryKey='';
function renderRunContext() {
 const r=snapshot.report,interrupted=$('run-interruptions'),ownership=$('owned-file-warning'),changed=snapshot.owned_changes||[];
 AC_I18N.bind(interrupted,()=>interruptionText(snapshot.interruptions));interrupted.hidden=!snapshot.interruptions?.length;
 ownership.hidden=!r.direct_workspace||r.plan.workspace_mode!=='shared'||(!changed.length&&r.state!=='running');
 AC_I18N.bind(ownership,()=>changed.length?acMessage('room.owned_edit_detected',{paths:changed.join(', ')}):acMessage('room.owned_edit_warning',{paths:r.plan.tasks.flatMap(t=>t.paths).join(', ')}));
 $('run-context').hidden=[...$('run-context').children].every(e=>e.hidden);
 $('run-diagnostics').hidden=$('run-context').hidden&&$('run-error').hidden;
}
function renderRunRecovery(r) {
 const broken=r.tests.find(x=>!(x.passed??(x.exit_code===0&&!x.timed_out))),failure=r.error||r.tasks.find(t=>t.status==='failed'&&t.error)?.error||'';
 const key=JSON.stringify([r.state,broken?.task,broken?.output]);if(key!==recoveryKey){recoveryKey=key;repairOpen=false;}
 document.querySelector('#run-screen .room-card').classList.toggle('repair-pending',coordinatorView&&!!broken&&!repairOpen);
 const why=broken?acMessage('room.integration_verification_failed_for',{value1:broken.task,value2:broken.output||acMessage('common.no_output')}):failure?acMessage('room.last_error')+String(failure):'';
 AC_I18N.bind($('run-error'),()=>why);$('run-error').hidden=!why;
 const scope=$('scope-review'),fix=$('fix-integration');scope.hidden=!scopeOffered(snapshot);scope.disabled=!idle();
 fix.hidden=!broken||repairOpen;fix.disabled=!idle();fix.onclick=()=>{repairOpen=true;fixIntegration(broken);renderRunRecovery(snapshot.report);};
 $('run-recovery-footer').hidden=scope.hidden&&fix.hidden;
 renderRunContext();
}
