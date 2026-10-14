// Generated, escaped HTML is fetched only when the offline manual is opened.
const AC_MANUAL=(()=>{
 const byId=id=>document.getElementById('start-manual-'+id),cache=new Map();let request,revision=0;
 async function render(language=AC_I18N.current()){const version=++revision,body=byId('body');body.replaceChildren();try{let html=cache.get(language);if(!html){html=(await request({kind:'manual_document',language})).html;cache.set(language,html);}if(version!==revision)return;const page=new DOMParser().parseFromString(html,'text/html');body.replaceChildren(...Array.from(page.body.childNodes,n=>document.importNode(n,true)));body.scrollTop=0;}catch(e){if(version===revision)body.textContent=e.message;}}
 byId('body').addEventListener('click',event=>{const link=event.target.closest('a');if(!link)return;const page=link.getAttribute('href');if(['user-manual.html','user-manual.ko.html'].includes(page)){event.preventDefault();render(page.includes('.ko.')?'ko':'en');}});
 function open(send){request=send;byId('dialog').showModal();return render();}
 document.addEventListener('ac-language-change',()=>{if(byId('dialog').open)render();});
 return {open,render};
})();
