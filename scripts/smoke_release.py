#!/usr/bin/env python3
"""Run exact packaged native binary acceptance against deterministic CLI fixtures."""
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def checked(command, cwd=None, evidence=None, name=None, denied=False):
    result=subprocess.run([str(arg) for arg in command],cwd=cwd,capture_output=True,text=True,encoding="utf-8",errors="replace",timeout=120)
    if evidence is not None:
        (evidence/(name+".stdout")).write_text(result.stdout, encoding="utf-8")
        (evidence/(name+".stderr")).write_text(result.stderr, encoding="utf-8")
    if denied:
        if result.returncode==0 or "approval" not in result.stderr.lower():
            raise RuntimeError("Unapproved merge was not refused for missing approval: "+result.stderr)
    elif result.returncode!=0:
        detail=""
        if len(command)>=4 and str(command[1])=="run":
            # Retain the run report so platform-specific failures are diagnosable from CI.
            status=subprocess.run([str(command[0]),"status",str(command[4])],capture_output=True,text=True,encoding="utf-8",errors="replace",timeout=60)
            detail="\nRun status:\n"+status.stdout[-8000:]+status.stderr[-2000:]
        raise RuntimeError("Command failed: "+str(command)+"\n"+result.stderr+detail)
    return result


def workflow_smoke(binary,worker,root,mode,folder=False):
    directory=root/(("folder-" if folder else "")+mode)
    directory.mkdir()
    repo=directory/"repo";repo.mkdir()
    evidence=directory/"evidence";evidence.mkdir()
    run=directory/"run"
    (repo/"base.txt").write_text("base\n", encoding="utf-8")
    base=None
    if not folder:
        checked(["git","init","-b","main"],cwd=repo)
        checked(["git","config","user.name","Release fixture"],cwd=repo)
        checked(["git","config","user.email","release@example.invalid"],cwd=repo)
        checked(["git","add","base.txt"],cwd=repo)
        checked(["git","commit","-m","base"],cwd=repo)
        base=checked(["git","rev-parse","HEAD"],cwd=repo).stdout.strip()
    paths=["one.txt","two.txt","three.txt"]
    plan={"goal":"Verify packaged native three-worker orchestration", "workspace_mode":mode,
          "agents":[{"id":"fixture","executable":str(worker),"args":["task"],"permission_mode":"external_sandbox"}],
          "tasks":[{"id":path,"agent":"fixture","prompt":path,"paths":[path],"test_command":[str(worker),"test"]} for path in paths]}
    plan_path=directory/"plan.json";plan_path.write_text(json.dumps(plan), encoding="utf-8")
    consent=json.loads(checked([binary,"validate",plan_path],evidence=evidence,name="validate").stdout)["plan_digest"]
    report=json.loads(checked([binary,"run",plan_path,repo,run,consent],evidence=evidence,name="run").stdout)
    if report["state"]!="waiting_approval" or any(t["status"]!="completed" for t in report["tasks"]):
        raise RuntimeError("Packaged workflow failed or did not wait for approval")
    if not report["tests"] or any(t["exit_code"]!=0 for t in report["tests"]):
        raise RuntimeError("Packaged workflow test evidence failed")
    ranges=[(t["started_ms"],t["finished_ms"]) for t in report["tasks"]]
    if max(t[0] for t in ranges)>=min(t[1] for t in ranges):
        raise RuntimeError("Three workers did not overlap")
    diff=checked([binary,"diff",run],evidence=evidence,name="diff").stdout
    if not all(path in diff for path in paths):
        raise RuntimeError("Diff omitted owned changes")
    # Ordinary folders run in the selected project with history kept outside it.
    if folder and (repo/".git").exists():
        raise RuntimeError("Original folder was initialized before approval")
    if not folder and checked(["git","rev-parse","HEAD"],cwd=repo).stdout.strip()!=base:
        raise RuntimeError("Target branch changed before approval")
    checked([binary,"merge",run],evidence=evidence,name="unapproved-merge",denied=True)
    candidate=report["candidate"]["commit"]
    checked([binary,"approve",run,candidate],evidence=evidence,name="approve")
    checked([binary,"merge",run],evidence=evidence,name="merge")
    if folder and (repo/".git").exists():
        raise RuntimeError("Original folder was initialized")
    if not folder and checked(["git","rev-parse","HEAD"],cwd=repo).stdout.strip()!=candidate:
        raise RuntimeError("Approved candidate was not merged")
    if not folder and checked(["git","status","--porcelain"],cwd=repo).stdout.strip():
        raise RuntimeError("Merged repository is dirty")
    if any((repo/path).read_text(encoding="utf-8")!="implemented\n" for path in paths):
        raise RuntimeError("Merged scoped content mismatch")
    return {"mode":mode,"source_mode":"folder" if folder else "git","base":base,"candidate":candidate,"parallel_ranges":ranges,
            "tasks":3,"tests":len(report["tests"]),"unapproved_merge_refused":True,"approved_merge_verified":True}


def run(package, output):
    root=Path(__file__).resolve().parents[1]
    executable="agentconductor.exe" if os.name=="nt" else "agentconductor"
    validator=root/"target"/"release"/executable
    validated=json.loads(checked([validator,"validate-package",package]).stdout)
    binary=Path(validated["binary"])
    manifest=validated["manifest"]
    output=Path(output).resolve();output.mkdir(parents=True,exist_ok=True)
    root=Path(tempfile.mkdtemp(prefix="ac-release-smoke-"))
    worker=root/("fixture.exe" if os.name=="nt" else "fixture")
    source=Path(__file__).resolve().parents[1]/"tests/support/release_worker.rs"
    checked(["rustc",source,"-o",worker])
    checked([binary,"--help"],evidence=output,name="help")
    result={"manifest":manifest,"workspace":str(root),"cases":[]}
    for mode in ["shared","worktree"]:
        for folder in [False,True]:
            result["cases"].append(workflow_smoke(binary,worker,root,mode,folder))
    result["exit_code"]=0
    (output/"smoke.json").write_text(json.dumps(result,indent=2)+"\n", encoding="utf-8")
    # Keep scoped workspaces and logs as evidence; no agent or GUI remains running.
    print(json.dumps(result))
    return result


if __name__=="__main__":
    if len(sys.argv)!=3:
        raise SystemExit("Usage: smoke_release.py <package-directory> <evidence-directory>")
    run(sys.argv[1],sys.argv[2])
