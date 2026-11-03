// One anchored selection surface for coordinator/role choices and existing select controls.
const selectionMenu=document.getElementById('assignment-menu');
let selectionAnchor=null,selectionConfig=null;
const nativeSelectionButtons=new Map();
function closeSelectionMenu(restore=false){selectionMenu.hidden=true;selectionAnchor?.setAttribute('aria-expanded','false');if(restore&&selectionAnchor?.isConnected)selectionAnchor.focus({preventScroll:true});}
function positionSelectionMenu(){
 const anchor=selectionAnchor.getBoundingClientRect(),frame=document.querySelector('.win').getBoundingClientRect();
 selectionMenu.style.width=Math.max(230,Math.min(310,anchor.width))+'px';selectionMenu.style.maxHeight=Math.max(120,Math.min(360,frame.height-24))+'px';
 const width=selectionMenu.offsetWidth,height=selectionMenu.offsetHeight;
 selectionMenu.style.left=Math.max(frame.left+8,Math.min(anchor.left,frame.right-width-8))+'px';
 selectionMenu.style.top=(anchor.bottom+height+6<=frame.bottom-8?anchor.bottom+6:Math.max(frame.top+8,anchor.top-height-6))+'px';
}
function openSelectionMenu(config){
 if(!selectionMenu.hidden&&selectionAnchor===config.anchor){closeSelectionMenu();return;}
 closeSelectionMenu();(config.anchor.closest('dialog[open]')||document.body).append(selectionMenu);selectionConfig=config;selectionAnchor=config.anchor;
 document.getElementById('assignment-menu-heading').textContent=config.title;
 const list=document.getElementById('assignment-options');list.replaceChildren();
 for(const option of config.options){
  const button=document.createElement('button');button.type='button';button.dataset.value=option.value;button.setAttribute('role','option');button.setAttribute('aria-selected',String(option.value===config.value));button.disabled=!!option.disabled;
  const text=document.createElement('span');text.textContent=option.label;const tick=document.createElement('span');tick.textContent=option.value===config.value?'✓':'';tick.setAttribute('aria-hidden','true');button.append(text,tick);
  button.onclick=()=>{if(option.custom){document.getElementById('assignment-manual-form').hidden=false;const input=document.getElementById('assignment-manual-model');input.value='';input.focus({preventScroll:true});positionSelectionMenu();}else{config.onSelect(option.value);closeSelectionMenu(true);}};list.append(button);
 }
 document.getElementById('assignment-manual-form').hidden=true;
 const note=document.getElementById('assignment-menu-note');note.textContent=config.note||'';note.hidden=!config.note;
 selectionMenu.hidden=false;selectionAnchor.setAttribute('aria-expanded','true');positionSelectionMenu();
 (list.querySelector('[aria-selected="true"]:not(:disabled)')||list.querySelector('button:not(:disabled)'))?.focus({preventScroll:true});
}
function syncSelectionButtons(){
 nativeSelectionButtons.forEach((button,select)=>{const option=select.selectedOptions[0];button.querySelector('span').textContent=option?(option.value==='claude'?'Claude Code':option.textContent):'선택';button.title=button.querySelector('span').textContent;button.disabled=select.disabled;});
}
document.querySelectorAll('select').forEach(select=>{
 const wrapper=document.createElement('div');wrapper.className='selection-control';select.before(wrapper);wrapper.append(select);select.hidden=true;select.classList.add('selection-source');
 const button=document.createElement('button');button.type='button';button.className='assignment-choice';button.setAttribute('aria-haspopup','listbox');button.setAttribute('aria-controls','assignment-options');button.setAttribute('aria-expanded','false');
 const label=select.getAttribute('aria-label')||select.closest('label')?.firstChild?.textContent?.trim()||'선택';button.setAttribute('aria-label',label);button.innerHTML='<span></span><span aria-hidden="true">⌄</span>';wrapper.append(button);nativeSelectionButtons.set(select,button);
 button.onclick=()=>openSelectionMenu({anchor:button,title:label,value:select.value,options:[...select.options].map(option=>({value:option.value,label:option.value==='claude'?'Claude Code':option.textContent,disabled:option.disabled})),onSelect:value=>{select.value=value;select.dispatchEvent(new Event('change',{bubbles:true}));syncSelectionButtons();}});
 select.addEventListener('change',syncSelectionButtons);new MutationObserver(syncSelectionButtons).observe(select,{childList:true,subtree:true,attributes:true,attributeFilter:['disabled','label','value','selected']});
});
document.getElementById('assignment-manual-form').onsubmit=event=>{event.preventDefault();const input=document.getElementById('assignment-manual-model'),value=input.value.trim();if(!value){input.reportValidity();return;}selectionConfig.onSelect(value);closeSelectionMenu(true);};
document.addEventListener('click',event=>{if(!selectionMenu.hidden&&!selectionMenu.contains(event.target)&&!selectionAnchor?.contains(event.target))closeSelectionMenu();});
document.addEventListener('keydown',event=>{
 if(selectionMenu.hidden)return;
 if(event.key==='Escape'){event.preventDefault();closeSelectionMenu(true);}
 if(event.key==='Tab')closeSelectionMenu(true);
 if((event.key==='ArrowDown'||event.key==='ArrowUp')&&event.target.closest('#assignment-options')){event.preventDefault();const items=[...document.querySelectorAll('#assignment-options button:not(:disabled)')],index=items.indexOf(document.activeElement);items[(index+(event.key==='ArrowDown'?1:-1)+items.length)%items.length].focus();}
});
window.addEventListener('resize',()=>closeSelectionMenu());window.addEventListener('scroll',event=>{if(!selectionMenu.contains(event.target))closeSelectionMenu();},true);syncSelectionButtons();
