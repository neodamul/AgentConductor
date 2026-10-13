// R-70 / DD-76: interface translation by exact English source text. User content
// (textareas, input values, logs, code) is never translated; nothing runs in English.
function acSwap(text,dict){const core=text.trim();if(!core)return text;if(Object.hasOwn(dict,core))return text.replace(core,dict[core]);const key=Symbol.for('acPrefixes'),prefixes=dict[key]||(dict[key]=Object.keys(dict).filter(k=>k.endsWith(' ')));for(const p of prefixes)if(core.startsWith(p))return text.replace(p,dict[p]);return text;}
function acInvert(dict){return Object.fromEntries(Object.entries(dict).map(([k,v])=>[v,k]));}
const AC_I18N=(()=>{
  const SKIP=new Set(['TEXTAREA','PRE','CODE','SCRIPT','STYLE']),ATTRS=['placeholder','title','aria-label'];
  const tables={ko:AC_KO},inverted={};
  let current='en',observer=null;
  function attrs(node,dict){for(const a of ATTRS){const v=node.getAttribute(a);if(v){const t=acSwap(v,dict);if(t!==v)node.setAttribute(a,t);}}}
  function walk(node,dict){
    if(node.nodeType===3){const t=acSwap(node.data,dict);if(t!==node.data)node.data=t;return;}
    if(node.nodeType!==1)return;
    // A textarea's text is user content; only its placeholder/title are interface.
    if(SKIP.has(node.tagName)||node.isContentEditable){if(node.tagName==='TEXTAREA')attrs(node,dict);return;}
    attrs(node,dict);
    for(const child of node.childNodes)walk(child,dict);
  }
  function skipped(node){for(let n=node.nodeType===1?node:node.parentNode;n&&n.nodeType===1;n=n.parentNode)if(SKIP.has(n.tagName)||n.isContentEditable)return true;return false;}
  function set(lang){
    if(lang!=='en'&&!tables[lang])lang='en';
    if(lang===current)return;
    if(observer)observer.disconnect();
    observer=null;
    if(current!=='en')walk(document.body,inverted[current]||(inverted[current]=acInvert(tables[current])));
    current=lang;
    document.documentElement.lang=lang;
    if(lang==='en')return;
    const dict=tables[lang];
    walk(document.body,dict);
    observer=new MutationObserver(list=>{for(const m of list){
      if(m.type==='childList'){if(!skipped(m.target))for(const n of m.addedNodes)walk(n,dict);}
      else if(m.type==='characterData'){if(!skipped(m.target)){const t=acSwap(m.target.data,dict);if(t!==m.target.data)m.target.data=t;}}
      else if(m.target.tagName==='TEXTAREA'||!skipped(m.target))attrs(m.target,dict);
    }});
    observer.observe(document.body,{subtree:true,childList:true,characterData:true,attributes:true,attributeFilter:ATTRS});
  }
  const saved=document.documentElement.lang||'en';
  document.documentElement.lang='en';
  set(saved);
  return {set,current:()=>current};
})();
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
