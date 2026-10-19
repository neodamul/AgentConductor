import subprocess,unittest
from pathlib import Path
class WorkspaceSwitchTests(unittest.TestCase):
 def test_an_old_workspace_snapshot_cannot_replace_the_new_workspace(self):
  source=Path('src/ui/startup.js').read_text(encoding='utf-8')
  start=source.index('async function poll(){');end=source.index("\n for(const id of ['name'",start);poll=source[start:end]
  script="const assert=require('node:assert/strict');let polling=false,home=true,workspaceEpoch=0,iconKinds='',draft={profiles:[]},state={config:{repository:'new',profiles:[]}},quotaRefreshed=new Map(),iconCache,profileStamp,queuedInitial=true,watchedJob=null,dirty=false,planDirty=false,composing=false,initialRouted=true,autoOpen=false,lastJob=null,mutating=false,planStarting=0;const PLAN_START_LIMIT=100,sources={wantsResult:()=>false,update:()=>{}},q=()=>({}),document={activeElement:null,querySelector:()=>({scrollIntoView(){}})},copy=x=>JSON.parse(JSON.stringify(x)),AC_I18N={bind:()=>{}},acMessage=()=>'',notice=()=>{},syncQuotaProject=()=>{},planProgress=()=>{},renderGoals=()=>{},autoDiscover=()=>{},profiles=()=>{},tasks=()=>{},renderButtons=()=>{},setTimeout=()=>0;let release;const request=()=>new Promise(r=>release=r);"+poll+";(async()=>{const pending=poll();workspaceEpoch++;const fresh={config:{repository:'new',profiles:[]}};state=fresh;release({config:{repository:'old',profiles:[]}});await pending;assert.equal(state,fresh);assert.equal(polling,false);})().catch(e=>{console.error(e);process.exitCode=1;});"
  subprocess.run(['node','-e',script],check=True)

 def test_a_closed_room_drops_its_inflight_snapshot(self):
  source=Path('src/ui/room.js').read_text(encoding='utf-8');start=source.index('async function poll() {');end=source.index('\nfor(const el',start);poll=source[start:end]
  script="const assert=require('node:assert/strict');let polling=false,roomStopped=false,snapshot={report:{plan:{agents:[]},tasks:[]}},iconKinds='',quotaRefreshed=new Map(),iconCache,lastJob=null,resumeAfter=null,review=null;let render=0,release;const request=()=>new Promise(r=>release=r),notice=()=>{},renderSnapshot=()=>render++,setTimeout=()=>0,autoQuotaRefresh=()=>{},loadTask=async()=>{},AC_I18N={bind:()=>{}},$=()=>({}),acMessage=()=>'';"+poll+";(async()=>{const original=snapshot;const pending=poll();roomStopped=true;release({report:{plan:{agents:[]},tasks:[]}});await pending;assert.equal(snapshot,original);assert.equal(render,0);})().catch(e=>{console.error(e);process.exitCode=1;});"
  subprocess.run(['node','-e',script],check=True)

if __name__=='__main__':unittest.main()
