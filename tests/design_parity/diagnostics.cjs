// Diagnostic-only evidence: never changes layout or a pass/fail threshold.
async function describe(browser, selector, index = 0) {
 const value = await browser.eval(`(() => {
  const e=[...document.querySelectorAll(${JSON.stringify(selector)})].filter(e=>e.getBoundingClientRect().width>0)[${index}];if(!e)return null;
  const style=n=>{const c=getComputedStyle(n);return Object.fromEntries(['fontFamily','fontSize','fontWeight','lineHeight','letterSpacing','display','alignItems','verticalAlign','padding','border','objectFit'].map(k=>[k,c[k]]));};
  const bounds=n=>{const r=n.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height,deviceX:r.x*devicePixelRatio,deviceY:r.y*devicePixelRatio};};
  const node=n=>n.nodeType===3?(()=>{const r=document.createRange();r.selectNodeContents(n);const b=r.getBoundingClientRect();return {tag:'#text',text:n.textContent.trim().slice(0,100),bounds:{x:b.x,y:b.y,width:b.width,height:b.height},style:style(n.parentElement)};})():{tag:n.tagName,text:n.textContent.trim().slice(0,100),bounds:bounds(n),style:style(n)};
  const pseudo=getComputedStyle(e,'::before');
  return {dpr:devicePixelRatio,element:node(e),children:[...e.childNodes].filter(n=>n.nodeType===1||n.nodeType===3).slice(0,8).map(node),parent:e.parentElement?node(e.parentElement):null,siblings:e.parentElement?[...e.parentElement.children].slice(0,8).map(c=>{const o=c.getAttribute('style');c.style.alignSelf='flex-start';const h=c.getBoundingClientRect().height;if(o===null)c.removeAttribute('style');else c.setAttribute('style',o);return {tag:c.tagName,id:c.id,text:c.textContent.trim().slice(0,40),height:c.getBoundingClientRect().height,intrinsicHeight:h,beforeHeight:getComputedStyle(c,'::before').height,font:getComputedStyle(c).fontFamily};}):[],before:{content:pseudo.content,width:pseudo.width,height:pseudo.height,maskImage:pseudo.maskImage?.slice(0,80)}};
 })()`);
 if(value&&(process.platform==='win32'||process.env.PARITY_FONT_DIAGNOSTICS==='1')){
  try{value.platformFonts=await browser.fonts(selector,index);}catch(error){value.platformFontError=error.message;}
 }
 return value;
}
module.exports={describe};
