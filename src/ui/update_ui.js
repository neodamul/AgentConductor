function acUpdateUi({q, request, notice}) {
  // DD-135: GitHub releases are checked in the background every 10 minutes; a newer one shows
  // the update icon left of Settings. The icon and the menu open the same update dialog.
  let update=null,updateWatch=null,updatePhase=null,updatePhaseAt=0;
  // GitHub #279: a check or an install counts from the moment this page first saw its phase.
  const updateSince=phase=>update?.phase===phase?' '+AC_ELAPSED(updatePhaseAt):'';
  function renderUpdate(){const u=update;if(!u)return;if(u.phase!==updatePhase){updatePhase=u.phase;updatePhaseAt=Date.now();}const b=q('update-button');b.hidden=!u.newer;b.classList.toggle('has-update',!!u.newer);const label=()=>(u.newer?acMessage("startup.update_available",{value1:u.latest}):acMessage("startup.update_check_button"));AC_I18N.bind(b,label,"title");AC_I18N.bind(b,label,"aria-label");
    AC_I18N.bind(q('update-title'),()=>(u.phase==='checking'?acMessage("startup.update_checking")+updateSince('checking'):u.newer?acMessage("startup.update_newer"):u.error?acMessage("startup.update_failed",{value1:u.error}):acMessage("startup.update_up_to_date")));
    AC_I18N.bind(q('update-versions'),()=>(acMessage("startup.update_installed_version",{value1:u.current})+(u.latest?' · '+acMessage("startup.update_latest_version",{value1:u.latest}):'')));
    AC_I18N.bind(q('update-state'),()=>(['installing','restarting'].includes(u.phase)?acMessage("startup.update_installing")+updateSince('installing'):u.newer&&u.error?acMessage("startup.update_failed",{value1:u.error}):u.newer&&!u.installable?acMessage("startup.update_unsupported"):''));
    q('update-notes').hidden=!u.page;AC_I18N.bind(q('update-notes'),()=>(u.page?acMessage("startup.update_release_notes")+': '+u.page:''));
    q('update-body').hidden=!u.notes;AC_I18N.bind(q('update-body'),u.notes||'');
    q('update-apply').hidden=!u.installable;q('update-apply').disabled=u.phase!=='idle';}
  async function updateRequest(kind){try{update=await request({kind});renderUpdate();}catch(e){if(q('update').open)notice(e.message,true);}}
  // Follows a check or an install until it settles (the app restarts after an install).
  function watchUpdate(){clearInterval(updateWatch);let tries=0;updateWatch=setInterval(async()=>{await updateRequest('update_status');if(++tries>400||(update?.phase==='idle'&&!q('update').open)){clearInterval(updateWatch);updateWatch=null;}},1500);}
  async function checkUpdates(){await updateRequest('check_update');watchUpdate();}
  async function openUpdate(){q('update').showModal();await checkUpdates();}
  q('update-button').onclick=()=>openUpdate();AC_I18N.bind(q('update-button'),()=>acMessage("startup.update_check_button"),'title');AC_I18N.bind(q('update-button'),()=>acMessage("startup.update_check_button"),'aria-label');
  q('update-apply').onclick=async()=>{try{update=await request({kind:'apply_update'});renderUpdate();watchUpdate();}catch(e){notice(e.message,true);}};
  checkUpdates();setInterval(checkUpdates,600000);
  return {openUpdate, refresh(){if(q('update').open&&['checking','installing'].includes(update?.phase))renderUpdate();}};
}
