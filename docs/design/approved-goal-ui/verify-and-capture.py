#!/usr/bin/env python3
"""Validate and render the approved DESIGN REFERENCE, never the application."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CHROME = Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
STATES = [
    ("01-workspace-goal", 0), ("02-agent-roles", 1),
    ("03-plan-review", 2), ("04-tasks", 3),
    ("05-tool-request", 4), ("06-permission-denied", 5),
    ("07-verification", 6), ("08-review-merge", 7),
    ("09-result", 8), ("10-new-goal", 9),
]
EXTRAS = ["inspector", "inspector-bottom", "goal-list", "delete-warning", "briefing", "chat",
          "settings", "quota-healthy", "quota-error", "quota-unsupported", "left-collapsed", "fullscreen", "coordinator-picker", "role-picker", "fallback-picker", "fallback-model-picker", "profile-edit", "profile-edit-bottom", "close-confirmation", "sessions", "sessions-confirm", "result-file-diff", "goal-title-open", "result-next-phase", "mcp", "mcp-add", "raw-log-multiline", "plan-history", "messenger-saving", "messenger-error", "goal-source-tags", "source-review-running", "source-review-done", "source-limit", "run-failure", "scope-failure", "claude-permissions"]



def contrast(a, b):
    def luminance(value):
        if len(value)==4:value='#'+''.join(c*2 for c in value[1:])
        c=[int(value[i:i+2],16)/255 for i in (1,3,5)]
        c=[x/12.92 if x<=0.04045 else ((x+0.055)/1.055)**2.4 for x in c]
        return sum(x*w for x,w in zip(c,[0.2126,0.7152,0.0722]))
    x,y=sorted([luminance(a),luminance(b)])
    return (y+0.05)/(x+0.05)


def palette_checks():
    css=(ROOT/'preview.css').read_text()
    # Later token blocks intentionally complete the original mock's light palette.
    light={}
    for block in re.findall(r':root\{([^}]*)\}',css):
        light.update(dict(re.findall(r'(--[\w-]+):\s*(#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{3}))\b',block)))
    dark=dict(light)
    for block in re.findall(r':root\[data-theme="dark"\]\{([^}]*)\}',css):
        dark.update(dict(re.findall(r'(--[\w-]+):\s*(#(?:[0-9a-fA-F]{6}|[0-9a-fA-F]{3}))\b',block)))
    output=[]
    for theme,tokens in [('light',light),('dark',dark)]:
        for fg,bg in [('--ink','--bg'),('--ink','--surface'),('--soft','--bg'),
                      ('--soft','--surface'),('--accent-ink','--accent'),
                      ('--warn','--warn-bg'),('--err','--err-bg'),('--role','--role-bg'),
                      ('--info','--info-bg'),('--ok','--ok-bg')]:
            ratio=contrast(tokens[fg],tokens[bg])
            assert ratio>=4.5,(theme,fg,bg,ratio)
            output.append({'theme':theme,'pair':[fg,bg],'contrast':round(ratio,2)})
    for theme,tokens in [('light',light),('dark',dark)]:
        for fg in ['--quota-short','--quota-long']:
            for bg in ['--bg','--surface']:
                ratio=contrast(tokens[fg],tokens[bg])
                assert ratio>=3,(theme,fg,bg,ratio)
                output.append({'theme':theme,'pair':[fg,bg],'contrast':round(ratio,2),'minimum':3})
    for theme,tokens in [('light',light),('dark',dark)]:
        for bg in ['--phase-prepare-bg','--phase-run-bg','--phase-finish-bg']:
            ratio=contrast(tokens['--soft'],tokens[bg])
            assert ratio>=4.5,(theme,bg,ratio)
            output.append({'theme':theme,'pair':['--soft',bg],'contrast':round(ratio,2)})
    return output


def main():
    source_files=['preview.html','preview.css','preview-state.js','preview-workspace.js','preview-controls.js',
                  'preview-tooltips.js','preview-selection.js','preview-assignment.js','preview-action-icons.js','preview-profile.js','preview-close.js','preview-evidence.js','preview-source-tags.js','preview-dialog-layout.js','preview-mcp.js','preview-sessions.js','preview-source-review.js']
    before={name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in source_files}
    report={'kind':'design-reference-only','approved_date':'2026-11-04',
            'viewport':[1280,960],'contrast':palette_checks(),'source_sha256':before,'captures':[]}
    jobs=[(name,view,'none') for name,view in STATES]+[(name,3,name) for name in EXTRAS]
    for theme in ['light','dark']:
        for name,view,mode in jobs:
            report['captures'].append({'file':'images/'+theme+'-'+name+'.png','theme':theme,'mode':mode,'view_index':view})
    (ROOT/'verification.json').write_text(json.dumps(report,separators=(',',':'))+'\n')
    subprocess.run(['rtk','proxy','node',str(ROOT/'verify-interactions.mjs')],check=True)
    assert before=={name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in source_files},'Preview changed during capture'
    # GitHub #849: the manifest records the bytes of the verified snapshot (every file but itself).
    manifest=json.loads((ROOT/'manifest.json').read_text())
    manifest['artifacts']={p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()
                           for p in sorted(ROOT.rglob('*')) if p.is_file() and p.name!='manifest.json'}
    (ROOT/'manifest.json').write_text(json.dumps(manifest,separators=(',',':'))+'\n')
    print('Verified',len(report['captures']),'captures and',len(report['contrast']),'contrast pairs.')


if __name__=='__main__':
    main()
