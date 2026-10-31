// GitHub #273: a simulator of the native bridge for checking what the operator sees at each step.
// Assembles the page the way src/ui/setup/document.rs does (without the CSP) and puts the
// scenario-driven fake bridge of sim.js in front of it. Usage:
//   node tests/ui_sim/build.cjs . <dir>/index.html && python3 -m http.server -d <dir> 8765
// then open http://127.0.0.1:8765/index.html, change SIM.home / SIM.room (or PRESET(...)) from the
// console and read the screen with PROBE() / ROOM().
const fs = require('fs'), path = require('path');
const [repo, out] = process.argv.slice(2), ui = f => fs.readFileSync(path.join(repo, 'src/ui', f), 'utf8');
const i18n = 'const AC_MESSAGES={en:' + ui('messages/en.json') + ',ko:' + ui('messages/ko.json') + '};\n' + ui('i18n.js');
const home = ui('startup.html') + ui('mcp.html') + ui('chat.html') + ui('optimize.html');
const style = [ui('room.css'), ui('startup.css'), ui('mcp.css'), ui('chat.css'), ui('optimize.css')].join('\n');
const script = [i18n, 'window.startRoom=function(){ ' + ui('room.js') + ' };', ui('mcp.js'), ui('chat.js'), ui('optimize.js'), ui('manual.js'), ui('startup.js')].join('\n');
const sim = fs.readFileSync(path.join(__dirname, 'sim.js'), 'utf8');
const page = ui('room.html').replace('<html lang="en">', '<html lang="ko" data-theme="light">')
  .replace('<link rel="stylesheet" href="/ui.css"><script src="/ui.js" defer></script>', '<style>' + style + '</style>')
  .replace('<body>', '<body><section id="startup-screen">' + home + '</section><section id="run-screen" hidden><button id="startup-home" class="small-button room-home"><span data-i18n="startup.workspace"></span></button>')
  .replace('</body>', '</section><script>' + sim + '</script><script>' + script + '</script></body>')
  .replace('{{APP_VERSION}}', 'sim');
fs.writeFileSync(out, page);
console.log('wrote', out, page.length);
