// #409: per-role draft state is separate from shared agent-profile fallback defaults.
(() => {
  let context=null,stamp='';
  const node=(tag,value)=>{const e=document.createElement(tag);if(typeof value==='function')AC_I18N.bind(e,value);else if(value!==undefined)e.textContent=value;return e;};
  const message=id=>acMessage(id);
  const profiles=()=>context.getProfiles();
  const profile=id=>profiles().find(p=>p.id===id);
  function options(select,items,value){
    select.replaceChildren(...items.map(([value,label,disabled])=>{const o=node('option',label);o.value=value;o.disabled=!!disabled;return o;}));
    select.value=items.some(([v,_,disabled])=>v===value&&!disabled)?value:'';
  }
  function models(select,agent,value){
    const p=profile(agent),ids=p?.catalog?.models||[];
    const values=['',...ids];
    if(select.dataset.models!==JSON.stringify(values)||select.dataset.modelAgent!==agent){
      options(select,[['',()=>message('choices.cli_default_model')],...ids.map(id=>[id,id])],value);
      select.dataset.models=JSON.stringify(values);
    }else select.value=values.includes(value)?value:'';
    select.dataset.modelAgent=agent||'';select.dataset.choiceIcon='model';select.disabled=!p;
    select.dataset.choiceManual=String(!!p&&(!ids.length||p.catalog?.origin==='manual'));
    select.acManualPick=async value=>{
      const owner=select.dataset.modelAgent;
      const catalog=await context.registerModel(owner,value);
      if(select.dataset.modelAgent!==owner)throw Error(message('choices.agent_changed'));
      options(select,[['',()=>message('choices.cli_default_model')],...catalog.models.map(id=>[id,id])],select.value);
      select.dataset.models=JSON.stringify(['',...catalog.models]);
    };
    AC_I18N.bind(select,()=>context.modelHint?.(select.dataset.modelAgent,select.value)||'','title');
  }
  function fallbackAgents(select,agent,value){
    const inherited=profile(agent)?.fallback;
    const suffix=inherited?.agent?': '+inherited.agent:'';
    const source=profile(agent),key=p=>p?.quota_group||p?.executable;
    const items=[[context.inheritToken,()=>message('choices.inherit')+suffix],['',()=>message('choices.none')],...profiles().filter(p=>p.id!==agent).map(p=>[p.id,p.id,!!source&&key(p)===key(source)])];
    const signature=JSON.stringify([agent,inherited?.agent,profiles().map(p=>[p.id,key(p)])]);
    if(select.dataset.agents!==signature){options(select,items,value);select.dataset.agents=signature;}
    select.dataset.choiceIcon='fallback';select.disabled=!agent;
  }
  function row(role,entries){
    const tr=node('tr');tr.dataset.role=role;
    const title=node('td',()=>context.labels[role]);AC_I18N.bind(title,()=>context.labels[role],'title');tr.append(title);
    const a=node('select'),m=node('select'),f=node('select'),fm=node('select');
    const fields=[['agent',a,'room.agent'],['model',m,'startup.model'],['fallback',f,'choices.fallback'],['fallback-model',fm,'startup.model']];
    for(const [name,s,label] of fields){s.dataset.roleField=name;s.id='start-role-'+role+'-'+name;AC_I18N.bind(s,()=>context.labels[role]+' · '+message(label),'aria-label');}
    a.dataset.choiceIcon='person';const primary=entries[0]?.agent||'',missing=!!primary&&!profile(primary);options(a,[['','—'],...profiles().map(p=>[p.id,p.id]),...(missing?[[primary,primary+' ⚠']]:[])],primary);a.classList.toggle('missing',missing);
    models(m,a.value,entries[0]?.model||'');
    const policy=entries[0]?.fallback,legacy=entries[1];
    const fallback=policy===false?'':policy?.agent||legacy?.agent||context.inheritToken;
    fallbackAgents(f,a.value,fallback);
    const effective=()=>f.value===context.inheritToken?profile(a.value)?.fallback?.agent||'':f.value;
    models(fm,effective(),policy?.model||legacy?.model||(fallback===context.inheritToken?profile(a.value)?.fallback?.model:'')||'');
    const changed=()=>{context.onChange();stamp=JSON.stringify(context.getRules());};
    a.onchange=()=>{
      a.classList.remove('missing');models(m,a.value,'');const before=f.value;
      fallbackAgents(f,a.value,before);
      models(fm,effective(),f.value===context.inheritToken?profile(a.value)?.fallback?.model||'':'');changed();
    };
    m.onchange=changed;
    f.onchange=()=>{models(fm,effective(),f.value===context.inheritToken?profile(a.value)?.fallback?.model||'':'');changed();};
    fm.onchange=()=>{if(f.value===context.inheritToken&&effective())f.value=effective();changed();};
    for(const s of [a,m]){const td=node('td');td.append(s);tr.append(td);}
    const arrow=node('td');arrow.className='role-fallback-arrow';arrow.setAttribute('aria-hidden','true');arrow.innerHTML='<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12h16M14 6l6 6-6 6"/></svg>';tr.append(arrow);
    for(const s of [f,fm]){const td=node('td');td.append(s);tr.append(td);}
    return tr;
  }
  function render(ctx){
    context=ctx;context.inheritToken='__inherit';while(profiles().some(p=>p.id===context.inheritToken))context.inheritToken+='_';const body=ctx.table.tBodies[0];
    const head=ctx.table.tHead.rows[0];if(head.cells.length===5){const arrow=node('th');arrow.setAttribute('aria-hidden','true');head.insertBefore(arrow,head.cells[3]);}
    const planning=node('tr');planning.className='role-planning';const name=node('td',()=>message('startup.planning')),description=node('td',()=>message('startup.planning_coordinator_workspace_settings'));description.colSpan=5;planning.append(name,description);
    body.replaceChildren(planning,...Object.keys(ctx.labels).map(role=>row(role,ctx.getRules()[role]||[])));
    stamp=JSON.stringify(ctx.getRules());
  }
  function read(table){
    const rules={};for(const tr of table.tBodies[0].querySelectorAll('tr[data-role]')){
      const field=name=>tr.querySelector('[data-role-field="'+name+'"]'),a=field('agent'),m=field('model'),f=field('fallback'),fm=field('fallback-model');
      if(!a?.value)continue;const entry={agent:a.value};if(m.value)entry.model=m.value;
      if(f.value==='')entry.fallback=false;else if(f.value!==context.inheritToken)entry.fallback={agent:f.value,...(fm.value?{model:fm.value}:{})};
      rules[tr.dataset.role]=[entry];
    }return rules;
  }
  function refresh(activeElement){
    if(!context)return;
    if(stamp!==JSON.stringify(context.getRules()))return render(context);
    for(const tr of context.table.tBodies[0].querySelectorAll('tr[data-role]')){
      const [a,m,f,fm]=tr.querySelectorAll('select');
      if(activeElement!==m&&activeElement!==window.acChoices?.buttonFor(m))models(m,a.value,m.value);
      fallbackAgents(f,a.value,f.value);
      const target=f.value===context.inheritToken?profile(a.value)?.fallback?.agent||'':f.value;
      const fallbackModel=f.value===context.inheritToken?profile(a.value)?.fallback?.model||'':fm.value;
      if(activeElement!==fm&&activeElement!==window.acChoices?.buttonFor(fm))models(fm,target,fallbackModel);
    }
  }
  window.acRoleChoices={render,read,refresh,models};
})();
