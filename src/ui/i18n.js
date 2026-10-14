// UI messages are addressed by stable IDs; only explicit bindings are localized.
const AC_I18N=(()=>{
  const bindings=new WeakMap();
  let current=Object.hasOwn(AC_MESSAGES,document.documentElement.lang)?document.documentElement.lang:'en';
  function format(id,params={},lang=current){
    const table=AC_MESSAGES[lang]||AC_MESSAGES.en;
    const template=Object.hasOwn(table,id)?table[id]:Object.hasOwn(AC_MESSAGES.en,id)?AC_MESSAGES.en[id]:id;
    return template.replace(/\{([a-zA-Z][a-zA-Z0-9_]*)\}/g,(token,key)=>Object.hasOwn(params,key)?String(params[key]):token);
  }
  function resolve(value){while(typeof value==='function')value=value();return value==null?'':String(value);}
  function write(node,value,attr){const text=resolve(value);if(attr){if(node.getAttribute(attr)!==text)node.setAttribute(attr,text);}else if(node.textContent!==text)node.textContent=text;return text;}
  function bind(node,value,attr=null){
    let slots=bindings.get(node);if(!slots){slots=new Map();bindings.set(node,slots);}
    if(node.nodeType===1)node.removeAttribute(attr?'data-i18n-'+attr:'data-i18n');
    if(typeof value==='function')slots.set(attr,value);else slots.delete(attr);
    return write(node,value,attr);
  }
  function replace(node,...children){
    const slots=bindings.get(node);if(slots)slots.delete(null);
    if(node.nodeType===1)node.removeAttribute('data-i18n');
    node.replaceChildren(...children);
  }
  function textNode(value){const node=document.createTextNode('');bind(node,value);return node;}
  function label(node,value){if(typeof value==='function')replace(node,textNode(value));else bind(node,value);}
  function walk(node){
    if(node.nodeType===1){
      const id=node.getAttribute('data-i18n');if(id)write(node,format(id),null);
      for(const attr of ['title','aria-label','placeholder']){const key=node.getAttribute('data-i18n-'+attr);if(key)write(node,format(key),attr);}
    }
    const slots=bindings.get(node);if(slots)for(const [attr,value] of slots)write(node,value,attr);
    for(const child of Array.from(node.childNodes||[]))walk(child);
  }
  function set(lang){
    const next=Object.hasOwn(AC_MESSAGES,lang)?lang:'en';
    if(next===current)return;
    current=next;document.documentElement.lang=current;
    walk(document.body);document.dispatchEvent(new Event('ac-language-change'));
  }
  document.documentElement.lang=current;walk(document.body);
  return {set,current:()=>current,format,bind,replace,label,text:(node,value)=>bind(node,value),node:textNode};
})();
function acMessage(id,params){return AC_I18N.format(id,params);}
// DD-111: Windows extended-length prefixes (\\?\C:\…, \\?\UNC\server\…, //?/C:/…) never
// reach the screen. Only rendered text and tooltips change; data values sent back to the
// engine, typed input and the Raw logs keep their exact text.
const AC_PATHS=(()=>{
  const fix=t=>t.replace(/\\\\\?\\UNC\\/gi,'\\\\').replace(/\\\\\?\\/g,'').replace(/\/\/\?\/UNC\//gi,'//').replace(/\/\/\?\//g,'');
  const has=t=>t.includes('\\\\?\\')||t.includes('//?/');
  const raw=n=>{for(let e=n.nodeType===1?n:n.parentNode;e&&e.nodeType===1;e=e.parentNode)if(e.tagName==='TEXTAREA'||e.tagName==='SCRIPT'||e.classList.contains('raw-log'))return true;return false;};
  const attrs=n=>{for(const a of ['title','aria-label']){const v=n.getAttribute(a);if(v&&has(v))n.setAttribute(a,fix(v));}};
  function walk(n){
    if(n.nodeType===3){if(has(n.data)&&!raw(n))n.data=fix(n.data);return;}
    if(n.nodeType!==1||raw(n))return;
    attrs(n);for(const c of n.childNodes)walk(c);
  }
  walk(document.body);
  new MutationObserver(list=>{for(const m of list){
    if(m.type==='childList')for(const n of m.addedNodes)walk(n);
    else if(m.type==='characterData')walk(m.target);
    else if(!raw(m.target))attrs(m.target);
  }}).observe(document.body,{subtree:true,childList:true,characterData:true,attributes:true,attributeFilter:['title','aria-label']});
  return {fix};
})();
// A pressed button shows that its work is running (user request 2026-10-14): it keeps its own
// colors with a moving bar and a progress cursor, so it never looks like a disabled button.
// track() covers a request; hold() keeps the mark while a background job started by it runs.
const AC_PRESS=(()=>{
  let last=null,lastAt=0;
  document.addEventListener('click',e=>{const b=e.target?.closest?.('button');if(b){last=b;lastAt=Date.now();}},true);
  function mark(b,on){if(!b?.classList)return;b.classList.toggle('is-busy',!!on);if(on)b.setAttribute('aria-busy','true');else b.removeAttribute('aria-busy');}
  function track(promise){const b=Date.now()-lastAt<2000?last:null;if(!b)return promise;mark(b,true);return Promise.resolve(promise).finally(()=>{if(!b.dataset.held)mark(b,false);});}
  function hold(b,on){if(!b?.classList)return;if(on)b.dataset.held='1';else delete b.dataset.held;mark(b,on);}
  return {track,hold};
})();
