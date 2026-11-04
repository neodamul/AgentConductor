// #479: goto() resolves only when the page it navigated to has loaded. A response whose body is held for longer than the old
// 10 s cap used to hand back a half-created document (documentElement null) and the next eval threw.
const http=require('http');
const {findChrome,launch}=require('./design_parity/cdp.cjs');const chrome=findChrome();if(!chrome){console.log('cdp goto skipped: no Chrome');process.exit(77);}
const HOLD_MS=11000;
const server=http.createServer((req,res)=>{res.writeHead(200,{'content-type':'text/html'});res.write(' ');setTimeout(()=>res.end('<!doctype html><html data-ready="yes"><body>slow</body></html>'),HOLD_MS);});
(async()=>{await new Promise(r=>server.listen(0,'127.0.0.1',r));const browser=await launch(chrome,{width:800,height:600});
 try{await browser.goto(`http://127.0.0.1:${server.address().port}/`);
  const ready=await browser.eval('document.documentElement.dataset.ready');
  if(ready!=='yes')throw Error('goto returned before the slow page loaded: '+ready);
  console.log('goto waits for a slow page');
 }finally{await browser.close();server.closeAllConnections?.();server.close();}})().catch(e=>{console.error(e);process.exitCode=1;});
