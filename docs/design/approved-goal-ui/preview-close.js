// Demonstrates R-118 without closing a real browser, app, or agent process.
function openClosePreview(){document.getElementById('close-run-dialog').showModal();document.querySelector('#close-run-dialog [value="keep"]').focus({preventScroll:true});}
document.getElementById('preview-close-window').onclick=()=>{closeAppMenu();openClosePreview();};
document.getElementById('close-run-dialog').addEventListener('close',event=>{
 document.documentElement.dataset.lastCloseChoice=event.target.returnValue||'cancel';
 document.querySelector('.mock-note').textContent='시안 · 창 닫기 선택: '+(event.target.returnValue==='keep'?'백그라운드에서 계속 실행':event.target.returnValue==='stop'?'실행 중지 후 닫기':'취소')+' · 실제 창이나 작업은 종료하지 않습니다.';
});
