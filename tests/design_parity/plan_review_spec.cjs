// #944: the Plan card header uses the approved mock, without another Phase box.
module.exports=[
 ...[
  ['eyebrow','.eyebrow','#start-plan-eyebrow'],
  ['title','h2','.step-title'],
 ].map(([name,mock,app])=>({name:'plan-review-head-'+name,props:['fontSize','fontWeight','color'],keys:['fontSize','fontWeight','color'],pixel:false,
  mock:{url:'?theme=light#2',selector:'.card[data-show="a"] .cardhead '+mock},
  app:{setup:"SIM.home.proposal=SIM.proposal('ready');",run:"document.getElementById('start-tab-review').click()",selector:'#start-panel-review '+app}})),
 {name:'plan-review-single-editable-list',assert:"document.querySelectorAll('#start-tasks .startup-task').length===3&&!document.querySelector('#start-plan-phases .plan-phase')&&!document.querySelector('#start-plan-phases .plan-phase-brief')&&/Phase 1\\/1.*Token rotation/.test(document.getElementById('start-plan-eyebrow').textContent)",
  app:{setup:"SIM.home.proposal=SIM.proposal('ready');SIM.home.proposal.slug='g';SIM.home.proposal.slug_phase='p1';SIM.home.slugs=[{slug:'g',goal:'g',active:true,plan:{phases:[{id:'p1',brief:'Tasks:\\n- task-1',specs:SIM.plan.tasks}]},progress:{stage:'phase',current_phase:'p1',phases:[{id:'p1',name:'Token rotation',stage:'plan',task:SIM.plan.tasks.map(t=>t.id),verification:[],completed:[]}]}}];",run:"document.getElementById('start-tab-review').click()"}},
];
