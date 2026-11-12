// #409: preserve native form values/events behind one event-driven anchored choice list.
(() => {
  const buttons=new WeakMap(),menu=document.createElement('div');
  menu.id='ac-choice-menu';menu.className='ac-choice-menu';menu.hidden=true;
  menu.innerHTML='<div class="ac-choice-heading"></div><div class="ac-choice-options" role="listbox" id="ac-choice-options"></div><form class="ac-choice-manual" hidden><label for="ac-choice-manual-model"></label><div><input id="ac-choice-manual-model" required maxlength="512"><button type="submit"></button></div><p role="alert" hidden></p></form>';
  document.body.append(menu);
  const heading=menu.querySelector('.ac-choice-heading'),list=menu.querySelector('.ac-choice-options'),form=menu.querySelector('form'),input=form.querySelector('input'),error=form.querySelector('p');
  let anchor=null,source=null;
  const text=id=>typeof AC_I18N!=='undefined'?acMessage(id):id;
  const display=(select,value,label)=>{const agent=select.dataset.choiceIcon==='person'||/coordinator$|provider$|agent$|agent-select$/.test(select.id);return agent&&value==='claude'?'Claude Code':agent&&value==='codex'?'Codex':label;};
  function close(restore=false){
    menu.hidden=true;anchor?.setAttribute('aria-expanded','false');
    if(restore&&anchor?.isConnected)anchor.focus({preventScroll:true});
  }
  function position(){
    if(menu.hidden||!anchor?.isConnected)return close();
    const a=anchor.getBoundingClientRect(),width=innerWidth,height=innerHeight;
    menu.style.width=Math.min(Math.max(230,Math.min(310,a.width)),width-16)+'px';
    menu.style.maxHeight=Math.max(80,Math.min(360,height-24))+'px';
    menu.style.left='0px';menu.style.top='0px';
    const origin=menu.getBoundingClientRect(),h=menu.offsetHeight,w=menu.offsetWidth;
    const x=Math.max(8,Math.min(a.left,width-w-8));
    const y=a.bottom+h+6<=height-8?a.bottom+6:Math.max(8,a.top-h-6);
    menu.style.left=x-origin.left+'px';menu.style.top=y-origin.top+'px';
  }
  function sync(select){
    const button=buttons.get(select);if(!button)return;
    const label=select.selectedOptions[0]?.textContent||'—';
    const shown=display(select,select.value,label);
    if(button.querySelector('.ac-choice-value').textContent!==shown)button.querySelector('.ac-choice-value').textContent=shown;
    button.disabled=select.disabled;button.title=select.title||shown;button.classList.toggle('missing',select.classList.contains('missing'));
    const labelText=select.getAttribute('aria-label')||[...select.labels||[]].map(e=>{const label=e.cloneNode(true);label.querySelectorAll('select,button,.ac-selection-control').forEach(n=>n.remove());return label.textContent.trim();}).join(' ')||select.id||text('choices.select');
    button.setAttribute('aria-label',labelText);const invalid=select.getAttribute('aria-invalid');if(invalid===null)button.removeAttribute('aria-invalid');else button.setAttribute('aria-invalid',invalid);
    const icons={person:'M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0 M4 21v-2a8 8 0 0 1 16 0v2',model:'M6 6h12v12H6z M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4',fallback:'M4 7h15l-4-4M20 17H5l4 4',sliders:'M4 7h16M4 17h16 M8 4v6M16 14v6'};
    const key=select.dataset.choiceIcon||(/model|모델/i.test(select.id+' '+labelText)?'model':/fallback|대체/i.test(select.id+' '+labelText)?'fallback':/agent|provider|에이전트|코디네이터|종류|계정/i.test(select.id+' '+labelText)?'person':'sliders');
    const svg='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="'+icons[key]+'"/></svg>';
    button.style.setProperty('--choice-icon','url("data:image/svg+xml,'+encodeURIComponent(svg)+'")');
  }
  function pick(value){
    const select=source;
    if(!select||select.disabled||!select.isConnected)return close();
    select.value=value;select.dispatchEvent(new Event('change',{bubbles:true}));sync(select);close(true);
  }
  function option(label,value,selected,disabled,action){
    const b=document.createElement('button');b.type='button';b.setAttribute('role','option');b.setAttribute('aria-selected',String(selected));b.dataset.value=value;b.disabled=disabled;
    const name=document.createElement('span'),tick=document.createElement('span');name.textContent=label;tick.textContent=selected?'✓':'';tick.setAttribute('aria-hidden','true');b.append(name,tick);b.onclick=action;list.append(b);
  }
  function open(select){
    const button=buttons.get(select);if(select.disabled||!button)return;
    if(!menu.hidden&&source===select)return close(true);
    close();sync(select);source=select;anchor=button;
    (select.closest('dialog[open]')||document.body).append(menu);
    heading.textContent=button.getAttribute('aria-label');list.replaceChildren();form.hidden=true;error.hidden=true;
    for(const o of select.options){
      const label=display(select,o.value,o.textContent);
      option(label,o.value,o.selected,o.disabled||o.parentElement?.disabled,()=>pick(o.value));
    }
    if(select.dataset.choiceManual==='true')option(text('choices.manual'),'__manual',false,false,()=>{
      form.hidden=false;form.querySelector('label').textContent=text('choices.model_id');form.querySelector('button').textContent=text('choices.apply');input.setAttribute('aria-label',text('choices.model_id'));input.placeholder=text('choices.model_placeholder');input.value='';input.focus({preventScroll:true});position();
    });
    menu.hidden=false;button.setAttribute('aria-expanded','true');position();
    (list.querySelector('[aria-selected="true"]:not(:disabled)')||list.querySelector('button:not(:disabled)'))?.focus({preventScroll:true});
  }
  function enhance(select){
    if(!select.isConnected)return;
    if(buttons.has(select)){sync(select);return;}
    const wrapper=document.createElement('div');wrapper.className='ac-selection-control';select.before(wrapper);wrapper.append(select);select.hidden=true;select.classList.add('ac-choice-source');
    const button=document.createElement('button');button.type='button';button.className='ac-choice-trigger';button.setAttribute('aria-haspopup','listbox');button.setAttribute('aria-controls',list.id);button.setAttribute('aria-expanded','false');
    button.innerHTML='<span class="ac-choice-value"></span><span class="ac-choice-chevron" aria-hidden="true">⌄</span>';wrapper.append(button);buttons.set(select,button);
    button.onclick=()=>open(select);button.onkeydown=e=>{if(['ArrowDown','ArrowUp',' '].includes(e.key)){e.preventDefault();open(select);}};sync(select);
  }
  function scan(root=document){
    if(root.matches?.('select'))enhance(root);
    root.querySelectorAll?.('select:not([data-native-choice])').forEach(enhance);
  }
  document.addEventListener('change',e=>{if(e.target.matches('select'))sync(e.target);});
  document.addEventListener('pointerdown',e=>{if(!menu.hidden&&!menu.contains(e.target)&&!anchor?.contains(e.target))close(true);},true);
  document.addEventListener('keydown',e=>{
    if(menu.hidden)return;
    if(e.key==='Escape'){e.preventDefault();e.stopPropagation();close(true);return;}
    if(e.key==='Tab'){close(true);return;}
    if(!list.contains(e.target))return;
    const items=[...list.querySelectorAll('button:not(:disabled)')],i=items.indexOf(document.activeElement);
    if(e.key==='ArrowDown'||e.key==='ArrowUp'){e.preventDefault();items[(i+(e.key==='ArrowDown'?1:-1)+items.length)%items.length]?.focus({preventScroll:true});}
    if(e.key==='Enter'||e.key===' '){e.preventDefault();document.activeElement.click();}
  },true);
  form.onsubmit=async e=>{
    e.preventDefault();const value=input.value.trim();
    if(!value||value.startsWith('-')||/\s/.test(value)){input.setCustomValidity(text('choices.invalid_model'));input.reportValidity();return;}
    input.setCustomValidity('');const select=source;form.querySelector('button').disabled=true;
    try{
      if(!select?.acManualPick)throw Error(text('choices.manual_unavailable'));
      await select.acManualPick(value);
      if(![...select.options].some(o=>o.value===value))select.add(new Option(value,value));
      if(source===select&&!menu.hidden)pick(value);
    }catch(e){if(source===select&&!menu.hidden){error.hidden=false;error.textContent=e.message;position();}}
    finally{form.querySelector('button').disabled=false;}
  };
  input.addEventListener('input',()=>input.setCustomValidity(''));
  document.addEventListener('close',e=>{if(e.target.contains(menu)){close();document.body.append(menu);}},true);
  new MutationObserver(records=>{
    for(const r of records){
      if(r.target.matches?.('select,option,optgroup')){const select=r.target.closest('select');if(select){if(source===select&&!menu.hidden&&form.hidden&&r.attributeName!=='aria-label')close(true);enhance(select);}}
      for(const n of r.addedNodes||[])if(n.nodeType===1)scan(n);
      if(r.target.matches?.('dialog'))scan(r.target);
    }
    if(!menu.isConnected){close();document.body.append(menu);}
    if(source&&!source.isConnected)close();
  }).observe(document.body,{childList:true,subtree:true,attributes:true,attributeFilter:['disabled','selected','label','value','open','aria-label']});
  window.addEventListener('resize',()=>close(true));
  window.addEventListener('scroll',e=>{if(!menu.contains(e.target))close();},true);
  window.acChoices={buttonFor:s=>buttons.get(s),open,close,sync:scan};scan();
})();
