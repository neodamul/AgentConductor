// #420: deterministic native-protocol state for the joined UI lifecycle; no CLI or filesystem call.
module.exports='('+function installLifecycle(){
 const clone=x=>JSON.parse(JSON.stringify(x));let revision=1,deletions=0;
 const bump=()=>{SIM.home.revision='flow-'+(++revision);};
 const goal=()=>SIM.home.slugs.find(g=>g.active);
 SIM.home.workspace_blank=true;SIM.home.config.goal='';SIM.home.config.workspace_mode='shared';
 SIM.home.config.roles={development:[{agent:'codex',model:'gpt'}]};
 SIM.home.config.profiles[0].coordinator_model='opus';
 SIM.reply.workspaces=()=>({name:SIM.home.config.name,current:SIM.home.root,registered:[{path:SIM.home.root,name:SIM.home.config.name}]});
 SIM.reply.create_workspace=p=>{SIM.home.workspace_blank=false;SIM.home.config.name=p.name;SIM.home.root='/tmp/lifecycle-workspace';bump();return {created:true};};
 SIM.reply.save=p=>{SIM.home.config=clone(p.config);bump();return {saved:true};};
 SIM.reply.baseline_status=()=>({source_mode:'folder',clean:true});
 SIM.reply.prepare=()=>{
  SIM.plan.goal=SIM.home.config.goal;SIM.plan.workspace_mode='shared';SIM.plan.agents=clone(SIM.home.config.profiles);
  SIM.home.proposal={...SIM.proposal('ready'),slug:'first_goal'};
  SIM.home.config.active_slug='first_goal';
  SIM.home.slugs=[{slug:'first_goal',goal:SIM.plan.goal,active:true,progress:{stage:'planned',phases:[]}}];
  bump();return {accepted:true};
 };
 SIM.reply.start=()=>{SIM.home.run_available=true;SIM.home.run_state='running';SIM.room.report.plan=clone(SIM.plan);bump();return {accepted:true};};
 SIM.reply.evidence=()=>({revision:SIM.room.revision,report:clone(SIM.room.report),diff:'diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -0,0 +1 @@\n+verified\n',truncated:false,test_output:[{task:'task-1',exit_code:0,timed_out:false,stdout:'passed',stderr:''}]});
 SIM.reply.goal_result=p=>({slug:p.slug,goal:SIM.home.slugs.find(g=>g.slug===p.slug).goal,report:clone(SIM.room.report),evidence:SIM.reply.evidence(),details:[]});
 SIM.reply.goal_folder=p=>({slug:p.slug,goal:SIM.home.slugs.find(g=>g.slug===p.slug).goal,root:'/tmp/sim-repo/'+p.slug,opened:!!p.open});
 SIM.reply.new_goal=()=>{SIM.home.slugs.forEach(g=>g.active=false);SIM.home.config.active_slug=null;SIM.home.config.goal='';SIM.home.proposal=null;SIM.home.run_available=false;SIM.home.run_state=null;bump();return {changed:true};};
 SIM.reply.delete_goal=p=>{if(p.revision!==SIM.home.revision)throw Error('Stale deletion revision');if(++deletions===1)return {deleted:false,stopping:true};SIM.home.slugs=SIM.home.slugs.filter(g=>g.slug!==p.slug);bump();return {deleted:true,changed:false};};
 SIM.reply.workspace_briefing=()=>({state:SIM.home.run_state,observed_at_ms:Date.now(),sessions:[],approvals:[]});
 SIM.reply.next_phase=()=>{SIM.plan.goal=SIM.home.config.goal;SIM.home.proposal={...SIM.proposal('ready'),slug:'second_goal',slug_phase:'02-next'};bump();return {accepted:true};};
 window.FLOW={
  complete(){SIM.room.report.state='completed';SIM.room.revision='completed';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.candidate={commit:'a'.repeat(40)};SIM.home.run_state='completed';goal().progress.stage='completed';bump();},
  phaseHistory(){SIM.home.config.active_slug='second_goal';SIM.home.slugs.push({slug:'second_goal',goal:SIM.home.config.goal,active:true,progress:{stage:'phase',current_phase:'02-next',phases:[{id:'01-core',name:'Core',stage:'completed',task:[],verification:[],completed:[]},{id:'02-next',name:'Next',stage:'task',task:[],verification:[],completed:[]}]},plan:{phases:[{id:'01-core',name:'Core',task:[],verification:[]},{id:'02-next',name:'Next',task:[],verification:[]}]}});bump();},
 };
 window.flowErrors=[];addEventListener('error',e=>flowErrors.push(e.message));addEventListener('unhandledrejection',e=>flowErrors.push(String(e.reason)));
}.toString()+')();';
