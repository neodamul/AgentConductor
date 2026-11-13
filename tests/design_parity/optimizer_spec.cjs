// #935/#936: independently styled reference lists and clarification controls.
const proposal={optimized:'리프레시 토큰 로테이션을 추가하고 테스트와 문서를 작성합니다.',changes:['검증 기준과 작업 범위를 명확히 했습니다.'],questions:['어떤 테스트 환경을 사용하나요?']};
const run=`(async()=>{const q=id=>document.getElementById('start-'+id),until=async f=>{for(let n=0;n<150;n++){if(f())return;await wait(40);}throw Error('optimizer parity not ready');};await until(()=>!q('optimize-button').disabled);SIM.reply.optimize_goal=()=>({id:'opt-parity'});q('optimize-button').click();await until(()=>SIM.log.some(x=>x.kind==='optimize_goal'));SIM.home.job={id:'opt-parity',kind:'optimize',state:'completed'};SIM.home.optimize={job:'opt-parity',proposal:${JSON.stringify(proposal)}};await until(()=>q('optimize').open);})()`;
module.exports=[
 ...['changes','questions'].map(id=>({name:'optimizer-'+id,props:['fontSize','lineHeight','color','marginTop','marginBottom','paddingLeft'],pixel:false,mock:{run:'openOptimizePreview()',selector:'#optimize-preview-'+id},app:{run,selector:'#start-optimize-'+id}})),
 {name:'optimizer-answer',props:['fontSize','lineHeight','paddingTop','paddingLeft','borderTopWidth','borderTopLeftRadius','backgroundColor'],pixel:false,mock:{run:'openOptimizePreview()',selector:'#optimize-preview-answers'},app:{run,selector:'#start-optimize-answers'}},
 {name:'optimizer-empty-answer-disabled',assert:"document.getElementById('start-optimize-again').disabled&&!document.getElementById('start-optimize-answers-section').hidden",app:{run}}
].map(s=>s.assert?s:{...s,keys:s.props});
