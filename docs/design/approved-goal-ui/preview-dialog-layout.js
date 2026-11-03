// Center reference dialogs inside the app frame, without transforming their popup scope.
function placeReferenceDialogs(){
 const frame=document.querySelector('.win').getBoundingClientRect();
 document.documentElement.style.setProperty('--reference-dialog-width',Math.max(0,frame.width-32)+'px');
 document.documentElement.style.setProperty('--reference-dialog-height',Math.max(0,frame.height-32)+'px');
 document.querySelectorAll('dialog[open]').forEach(dialog=>{const box=dialog.getBoundingClientRect();dialog.style.left=(frame.left+(frame.width-box.width)/2)+'px';dialog.style.top=(frame.top+(frame.height-box.height)/2)+'px';});
}
const referenceDialogResize=new ResizeObserver(placeReferenceDialogs);referenceDialogResize.observe(document.querySelector('.win'));
const referenceDialogOpen=new MutationObserver(placeReferenceDialogs);
document.querySelectorAll('dialog').forEach(dialog=>{referenceDialogResize.observe(dialog);referenceDialogOpen.observe(dialog,{attributes:true,attributeFilter:['open']});});placeReferenceDialogs();
