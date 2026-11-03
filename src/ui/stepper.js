// GitHub #390 / DD-222: one six-step stepper (Workspace, Agent roles, Plan, Tasks, Verify, Review and
// merge) shared by the Workspace screen and the Agent Room. Steps 1-3 stay in startup.js (showStep);
// this file owns steps 4-6, the Room's stage marks and moving the one nav between the two screens.
window.acStepper=(()=>{
 const runSteps=['tasks','verify','merge'],prepareSteps=['workspace','team','review'],
  stageStep={plan:'tasks',task:'tasks',verify:'verify',merge:'merge',complete:'merge'};
 const tab=name=>document.getElementById('start-tab-'+name),nav=()=>document.getElementById('start-step-tabs');
 let home;
 return{
  runSteps,
  stepOf:stage=>stageStep[stage],
  // The Room marks the step of the run's stage; steps before it are done and a merge in progress pulses.
  // A null stage clears the marks so the Workspace screen's own selection shows again.
  paint(stage,running=false){
   // The Room keeps polling after the operator goes back to the Workspace; it must not re-mark the stepper then.
   if(stage&&!document.body.classList.contains('on-run'))return;
   const current=stageStep[stage],at=runSteps.indexOf(current);
   for(const name of prepareSteps){const t=tab(name);t.classList.toggle('done',at>=0);if(at>=0){t.setAttribute('aria-selected','false');t.tabIndex=-1;}}
   for(const name of runSteps){const t=tab(name),index=runSteps.indexOf(name),selected=name===current;t.setAttribute('aria-selected',String(selected));t.tabIndex=selected?0:-1;t.classList.toggle('done',index<at);t.classList.toggle('running',running&&selected);}
  },
  // Steps 4-6 open the Room, so they exist only while the workspace has a run to open.
  available(on){for(const name of runSteps)tab(name).disabled=!on;},
  // The Room shows the stepper under its goal bar; the Workspace screen gets it back from `restore`.
  place(host,before){if(!home){home=document.createComment('stepper');nav().before(home);}host.insertBefore(nav(),before);},
  restore(){if(home?.parentNode)home.parentNode.insertBefore(nav(),home);},
  bind({enter,home:goHome,show}){
   nav().addEventListener('click',event=>{
    const target=event.target.closest?.('[role="tab"]');if(!target||target.disabled)return;
    const name=target.id.replace('start-tab-',''),onRun=document.body.classList.contains('on-run');
    if(runSteps.includes(name)){if(!onRun)enter();}else if(onRun)goHome().then(ok=>ok&&show(name));
   });
  }
 };
})();
