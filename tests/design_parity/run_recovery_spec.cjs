// #927: failure evidence and recovery stay in the approved task card, with no status banner.
const setup="SIM.home.run_available=true;SIM.home.run_state='failed';SIM.room.report.state='failed';SIM.room.report.tasks.forEach(t=>t.status='completed');SIM.room.report.tests=[{task:'task-1',exit_code:1,timed_out:false,output:'assertion failed: expected token rotation'}];";
const run="document.getElementById('start-tab-tasks').click()";
module.exports=[
 {name:'run-failure-no-banner',assert:"!document.getElementById('run-banner')&&!!document.querySelector('.room-card #run-error')&&!document.getElementById('run-error').hidden&&!document.getElementById('fix-integration').hidden",app:{setup,run}},
 {name:'run-failure-evidence',props:['fontSize','paddingTop','paddingLeft','borderTopLeftRadius','whiteSpace','overflowWrap','maxHeight','overflowY','backgroundColor','color'],pixel:false,mock:{url:'?theme=light#6',run:'previewRunFailure()',selector:'.preview-run-error'},app:{setup,run,selector:'#run-error'}},
 {name:'run-failure-repair',props:['fontSize','borderTopLeftRadius','backgroundColor','color'],pixel:false,mock:{url:'?theme=light#6',run:'previewRunFailure()',selector:'[data-recovery=test]'},app:{setup,run,selector:'#fix-integration'}},
 {name:'run-scope-no-banner',assert:"!document.getElementById('run-banner')&&!document.getElementById('scope-review').hidden&&!!document.querySelector('.room-card #scope-review')",app:{setup:setup+"SIM.room.report.tests=[];SIM.room.report.error='out-of-scope change: outside.txt';",run}}
].map(s=>s.assert?s:{...s,keys:s.name==='run-failure-repair'?[...s.props,'height']:s.props});
