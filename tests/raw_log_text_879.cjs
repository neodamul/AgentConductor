// #879: decode JSON string newlines for display, never literal path escapes or source artifacts.
const assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const format=vm.runInNewContext(fs.readFileSync('src/ui/room_activity.js','utf8')+'\nreadableRawLog;',{});
const record={type:'assistant',message:{content:[{type:'text',text:'first line\nsecond line\n<b>literal</b>',path:'C:\\new\\test',literal:'\\n stays literal'}]},usage:{input_tokens:12}};
const source=JSON.stringify(record)+'\n'+JSON.stringify({type:'result',result:'done\nnext'})+'\n';
const shown=format(source);
assert(shown.includes('first line\nsecond line\n<b>literal</b>'));
assert(shown.includes('done\nnext'));
assert(shown.includes(JSON.stringify(record.message.content[0].path)));
assert(shown.includes(JSON.stringify(record.message.content[0].literal)));
assert(shown.includes('"input_tokens": 12')&&shown.includes('"type": "result"'));
assert.equal(source,JSON.stringify(record)+'\n'+JSON.stringify({type:'result',result:'done\nnext'})+'\n');
for(const plain of ['plain first\nplain second\n','partial {"text":\n','42\nnull\n','error C:\\new\\test\\n',''])assert.equal(format(plain),plain);
const clipped='tail fragment\n'+JSON.stringify({message:'visible\nline'})+'\nmalformed {';
assert(format(clipped).startsWith('tail fragment\n')&&format(clipped).endsWith('\nmalformed {'));
console.log('raw log text: nested records, newlines, metadata, literal escapes, plain/malformed tails passed');
