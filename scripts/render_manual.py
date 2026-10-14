#!/usr/bin/env python3
"""Build offline manuals from the Markdown subset used by this repository; no dependencies."""
import base64
import html
import re
from pathlib import Path
from urllib.parse import urlsplit


def render(text, directory):
    directory = Path(directory).resolve()
    def inline(value):
        out=[]; last=0
        pattern=r'`([^`]+)`|\*\*([^*]+)\*\*|(!?)\[([^]]*)\]\(([^)]+)\)'
        for match in re.finditer(pattern,value):
            out.append(html.escape(value[last:match.start()])); last=match.end()
            code,strong,image,label,url=match.groups()
            if code is not None: out.append('<code>'+html.escape(code)+'</code>');continue
            if strong is not None: out.append('<strong>'+html.escape(strong)+'</strong>');continue
            if image:
                path=(directory/url).resolve()
                if not path.is_relative_to(directory/'assets') or path.suffix.lower()!='.png':
                    raise ValueError('Manual images must be local PNG assets: '+url)
                data=base64.b64encode(path.read_bytes()).decode('ascii')
                out.append('<img alt="'+html.escape(label,quote=True)+'" src="data:image/png;base64,'+data+'">')
            elif urlsplit(url).scheme not in ('','https','http','mailto') or url.startswith('//'):
                out.append(html.escape(label))
            else:
                if url in ('user-manual.md','user-manual.ko.md'):url=url[:-3]+'.html'
                out.append('<a target="_blank" rel="noopener" href="'+html.escape(url,quote=True)+'">'+html.escape(label)+'</a>')
        out.append(html.escape(value[last:]));return ''.join(out)
    def cells(line):return [v.strip().replace(r'\|','|') for v in re.split(r'(?<!\\)\|',line.strip().strip('|'))]
    lines=text.splitlines();out=[];i=0
    while i<len(lines):
        line=lines[i];i+=1
        if not line.strip():continue
        if line.startswith('```'):
            code=[]
            while i<len(lines) and not lines[i].startswith('```'):code.append(lines[i]);i+=1
            if i<len(lines):i+=1
            out.append('<pre><code>'+html.escape('\n'.join(code))+'</code></pre>');continue
        heading=re.match(r'^(#{1,6})\s+(.*)',line)
        if heading:
            n=len(heading[1]);out.append(f'<h{n}>'+inline(heading[2])+f'</h{n}>');continue
        if line.startswith('|') and i<len(lines) and all(re.fullmatch(r':?-+:?',v) for v in cells(lines[i])):
            out.append('<div class="manual-table"><table><thead><tr>'+''.join('<th>'+inline(v)+'</th>' for v in cells(line))+'</tr></thead><tbody>');i+=1
            while i<len(lines) and lines[i].startswith('|'):
                out.append('<tr>'+''.join('<td>'+inline(v)+'</td>' for v in cells(lines[i]))+'</tr>');i+=1
            out.append('</tbody></table></div>');continue
        if re.match(r'^\s*- ',line):
            out.append('<ul>')
            while True:
                item=[re.sub(r'^\s*- ','',line)]
                while i<len(lines) and lines[i].startswith('  ') and not re.match(r'^\s*- ',lines[i]):item.append(lines[i].strip());i+=1
                out.append('<li>'+inline(' '.join(item))+'</li>')
                if i>=len(lines) or not re.match(r'^\s*- ',lines[i]):break
                line=lines[i];i+=1
            out.append('</ul>');continue
        paragraph=[line]
        while i<len(lines) and lines[i].strip() and not re.match(r'^(#|\||```|\s*- )',lines[i]):paragraph.append(lines[i]);i+=1
        out.append('<p>'+inline(' '.join(paragraph))+'</p>')
    lang='ko' if '사용' in lines[0] else 'en'
    style='body{font:14px/1.65 system-ui,sans-serif;max-width:1000px;margin:24px auto;padding:0 20px;overflow-wrap:anywhere}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ccc;padding:8px;text-align:left;vertical-align:top}th{background:#eee}pre{background:#f5f5f5;padding:12px;white-space:pre-wrap}img{max-width:100%;height:auto}.manual-table{overflow:auto}'
    return '<!doctype html>\n<html lang="'+lang+'"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>AgentConductor User Manual</title><style>'+style+'</style></head><body>\n'+'\n'.join(out)+'\n</body></html>\n'


if __name__=='__main__':
    root=Path(__file__).resolve().parents[1]/'docs'
    for name in ['user-manual','user-manual.ko']:
        source=root/(name+'.md')
        (root/(name+'.html')).write_text(render(source.read_text(encoding='utf-8'),root),encoding='utf-8')
