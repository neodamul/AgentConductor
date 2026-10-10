#!/usr/bin/env python3
"""Run exact packaged native binary acceptance against deterministic CLI fixtures."""
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path


def validated_binary(package):
    package=Path(package).resolve()
    manifest=json.loads((package/"manifest.json").read_text())
    name=manifest["binary"]
    if name not in ["agentconductor","agentconductor.exe"]:
        raise ValueError("Invalid packaged binary path")
    binary=package/name
    if binary.is_symlink() or not binary.is_file():
        raise ValueError("Packaged binary must be a regular file")
    checksum=hashlib.sha256(binary.read_bytes()).hexdigest()
    if checksum!=manifest["binary_sha256"]:
        raise ValueError("Packaged binary checksum mismatch")
    if not re.fullmatch(r"[0-9a-f]{40}",manifest["source_commit"]):
        raise ValueError("Exact packaged source commit required")
    if (package/"SHA256SUMS").read_text()!=checksum+"  "+name+"\n":
        raise ValueError("Package checksum file mismatch")
    return binary,manifest


def checked(command, cwd=None, evidence=None, name=None, denied=False):
    result=subprocess.run([str(arg) for arg in command],cwd=cwd,capture_output=True,text=True,timeout=120)
    if evidence is not None:
        (evidence/(name+".stdout")).write_text(result.stdout)
        (evidence/(name+".stderr")).write_text(result.stderr)
    if denied:
        if result.returncode==0 or "approval" not in result.stderr.lower():
            raise RuntimeError("Unapproved merge was not refused for missing approval: "+result.stderr)
    elif result.returncode!=0:
        detail=""
        if len(command)>=4 and str(command[1])=="run":
            # Retain the run report so platform-specific failures are diagnosable from CI.
            status=subprocess.run([str(command[0]),"status",str(command[4])],capture_output=True,text=True,timeout=60)
            detail="\nRun status:\n"+status.stdout[-8000:]+status.stderr[-2000:]
        raise RuntimeError("Command failed: "+str(command)+"\n"+result.stderr+detail)
    return result


def workflow_smoke(binary,worker,root,mode):
    directory=root/mode
    directory.mkdir()
    repo=directory/"repo";repo.mkdir()
    evidence=directory/"evidence";evidence.mkdir()
    run=directory/"run"
    checked(["git","init","-b","main"],cwd=repo)
    checked(["git","config","user.name","Release fixture"],cwd=repo)
    checked(["git","config","user.email","release@example.invalid"],cwd=repo)
    (repo/"base.txt").write_text("base\n")
    checked(["git","add","base.txt"],cwd=repo)
    checked(["git","commit","-m","base"],cwd=repo)
    base=checked(["git","rev-parse","HEAD"],cwd=repo).stdout.strip()
    paths=["one.txt","two.txt","three.txt"]
    plan={"goal":"Verify packaged native three-worker orchestration", "workspace_mode":mode,
          "agents":[{"id":"fixture","executable":str(worker),"args":["task"],"permission_mode":"external_sandbox"}],
          "tasks":[{"id":path,"agent":"fixture","prompt":path,"paths":[path],"test_command":[str(worker),"test"]} for path in paths]}
    plan_path=directory/"plan.json";plan_path.write_text(json.dumps(plan))
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
    if checked(["git","rev-parse","HEAD"],cwd=repo).stdout.strip()!=base:
        raise RuntimeError("Target branch changed before approval")
    checked([binary,"merge",run],evidence=evidence,name="unapproved-merge",denied=True)
    candidate=report["candidate"]["commit"]
    checked([binary,"approve",run,candidate],evidence=evidence,name="approve")
    checked([binary,"merge",run],evidence=evidence,name="merge")
    if checked(["git","rev-parse","HEAD"],cwd=repo).stdout.strip()!=candidate:
        raise RuntimeError("Approved candidate was not merged")
    if checked(["git","status","--porcelain"],cwd=repo).stdout.strip():
        raise RuntimeError("Merged repository is dirty")
    if any((repo/path).read_text()!="implemented\n" for path in paths):
        raise RuntimeError("Merged scoped content mismatch")
    return {"mode":mode,"base":base,"candidate":candidate,"parallel_ranges":ranges,
            "tasks":3,"tests":len(report["tests"]),"unapproved_merge_refused":True,"approved_merge_verified":True}


def run(package, output):
    binary,manifest=validated_binary(package)
    output=Path(output).resolve();output.mkdir(parents=True,exist_ok=True)
    root=Path(tempfile.mkdtemp(prefix="ac-release-smoke-"))
    worker=root/("fixture.exe" if os.name=="nt" else "fixture")
    source=Path(__file__).resolve().parents[1]/"tests/support/release_worker.rs"
    checked(["rustc",source,"-o",worker])
    checked([binary,"--help"],evidence=output,name="help")
    result={"manifest":manifest,"workspace":str(root),"cases":[]}
    for mode in ["shared","worktree"]:
        result["cases"].append(workflow_smoke(binary,worker,root,mode))
    result["exit_code"]=0
    (output/"smoke.json").write_text(json.dumps(result,indent=2)+"\n")
    # Keep scoped workspaces and logs as evidence; no agent or GUI remains running.
    print(json.dumps(result))
    return result


if __name__=="__main__":
    if len(sys.argv)!=3:
        raise SystemExit("Usage: smoke_release.py <package-directory> <evidence-directory>")
    run(sys.argv[1],sys.argv[2])
