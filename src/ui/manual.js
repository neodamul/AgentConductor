// DD-147: offline bilingual manual; source text never becomes executable HTML.
const AC_MANUAL=(()=>{
 const byId=id=>document.getElementById('start-manual-'+id);
 function inline(node,text){const pattern=/\*\*([^*]+)\*\*|\x60([^\x60]+)\x60/g;let last=0;for(const match of text.matchAll(pattern)){node.append(document.createTextNode(text.slice(last,match.index)));const span=document.createElement(match[1]?'strong':'code');span.textContent=match[1]||match[2];node.append(span);last=match.index+match[0].length;}node.append(document.createTextNode(text.slice(last)));}
 function render(){const body=byId('body');body.replaceChildren();const text=AC_MANUAL_TEXT[AC_I18N.current()]||AC_MANUAL_TEXT.en;let code=null,list=null;for(const line of text.split('\n')){if(line.startsWith('\x60\x60\x60')){list=null;if(code){code=null;}else{code=document.createElement('pre');body.append(code);}continue;}if(code){code.append(document.createTextNode(line+'\n'));continue;}if(!line.trim()){list=null;continue;}const heading=line.match(/^(#{1,6})\s+(.*)/);if(heading){list=null;const h=document.createElement('h'+heading[1].length);inline(h,heading[2]);body.append(h);continue;}if(/^\s*- /.test(line)){if(!list){list=document.createElement('ul');body.append(list);}const item=document.createElement('li');inline(item,line.replace(/^\s*- /,''));list.append(item);continue;}list=null;const p=document.createElement('p');inline(p,line);body.append(p);}body.scrollTop=0;}
 function open(){render();byId('dialog').showModal();}
 document.addEventListener('ac-language-change',()=>{if(byId('dialog').open)render();});
 return {open,render};
})();
