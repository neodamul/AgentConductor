// #468: the About dialog opens once per installed version (first start after an install or update) and from the menu.
window.acAboutSetup=request=>{
  const q=id=>document.getElementById(id),dialog=q('about-dialog');
  function show(info,updated){
    AC_I18N.bind(q('about-meta'),()=>acMessage('about.meta',{version:info.version}));
    const tag=q('about-tag');tag.hidden=!updated;if(updated)AC_I18N.bind(tag,()=>acMessage('about.updated',{version:info.version}));
    if(!dialog.open)dialog.showModal();q('about-start').focus({preventScroll:true});
  }
  async function open(updated=false){const info=await request({kind:'about_status'});show(info,updated);}
  function close(){if(dialog.open)dialog.close();}
  q('about-close').onclick=q('about-start').onclick=close;
  // Closing by any route (button, Escape) records the version, so the next start stays quiet.
  dialog.addEventListener('close',()=>{request({kind:'about_seen'}).catch(()=>{});});
  dialog.querySelectorAll('.about-copy').forEach(button=>{button.onclick=async()=>{
    try{await navigator.clipboard.writeText(button.parentElement.querySelector('code').textContent);}catch{return;}
    AC_I18N.bind(button,()=>acMessage('about.copied'));setTimeout(()=>AC_I18N.bind(button,()=>acMessage('about.copy')),1200);};});
  window.acAbout={open:()=>open(false),first:async()=>{const info=await request({kind:'about_status'});if(info.show)show(info,info.updated);}};
};
