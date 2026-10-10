"""Controlled non-steerable CLI fixture; uses only explicit checkpoint commands."""
import json
import os
import pathlib
import subprocess
import sys
import time

prompt = sys.stdin.read()
run = pathlib.Path(os.environ["AC_CHECKPOINT_RUN_DIR"])
address_file = pathlib.Path(os.environ["AC_CHECKPOINT_ADDRESS_FILE"])
address = json.loads(address_file.read_text())
assert address["session_id"] == sys.argv[1]
assert "Cooperative checkpoint contract" in prompt
resumed_rework = address["attempt"] > 1 and os.environ.get("MOCK_REQUIRE_CONTINUATION")
if resumed_rework:
    assert '"instruction": "Use revised content"' in prompt
    assert '"source": "local checkpoint ledger"' in prompt
    assert pathlib.Path(address["task"]).read_text() == "partial\n"
binary = os.environ["AC_CHECKPOINT_EXECUTABLE"]
artifacts = address_file.parent


def command(name, packet):
    path = artifacts / (name + ".json")
    path.write_text(json.dumps(packet))
    result = subprocess.run([binary, name, str(run), str(path)], capture_output=True, text=True, timeout=5)
    assert result.returncode == 0, result.stderr
    return json.loads(result.stdout)


def report(sequence, phase):
    command("checkpoint-report", dict(address=address, sequence=sequence,
            reported_at_ms=time.time_ns() // 1_000_000, phase=phase,
            summary="Checkpoint fixture reports its real activity", next_step="Inspect addressed directives"))


assert command("checkpoint-poll", address) == []
report(1, "prepared")
(artifacts / "checkpoint-ready").write_text("ready")
deadline = time.monotonic() + 12
while not pathlib.Path(os.environ["MOCK_RELEASE"]).exists():
    assert time.monotonic() < deadline
    time.sleep(0.02)
directives = command("checkpoint-poll", address)
content = "revised\n" if resumed_rework else "implemented\n"
for directive in directives:
    assert directive["address"] == address
    assert directive["action"] in ("rework", "pause", "handoff")
    assert directive["instruction"] == ("Use revised content" if directive["action"] == "rework" else "Relinquish at checkpoint")
    packet = dict(id=directive["id"], address=address, state="accepted",
                  reported_at_ms=time.time_ns() // 1_000_000, reason="Safe checkpoint reached")
    ack_mode = os.environ.get("MOCK_ACK_MODE", "completed")
    if ack_mode != "pending":
        command("checkpoint-ack", packet)
    content = "revised\n" if directive["action"] == "rework" else "partial\n"
    packet.update(state="completed", reported_at_ms=time.time_ns() // 1_000_000)
    if ack_mode == "completed":
        command("checkpoint-ack", packet)
    if directive["action"] != "rework" or os.environ.get("MOCK_REQUIRE_CONTINUATION"):
        if directive["action"] == "rework":
            content = "partial\n"
            pathlib.Path(address["task"]).write_text(content)
        (artifacts / "checkpoint-relinquished").write_text("acknowledged, process still live")
        while not pathlib.Path(os.environ["MOCK_EXIT_RELEASE"]).exists():
            assert time.monotonic() < deadline
            time.sleep(0.02)
pathlib.Path(address["task"]).write_text(content)
report(2, "completed")
session = "foreign-session" if os.environ.get("MOCK_BAD_SESSION") else address["session_id"]
print(json.dumps(dict(type="session", session_id=session)), flush=True)
print(json.dumps(dict(type="result", payload=dict(summary="Checkpoint fixture completed"))), flush=True)
