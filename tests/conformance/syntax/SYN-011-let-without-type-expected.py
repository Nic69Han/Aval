#!/usr/bin/env python3
# scenario: SYN-011
import json, os, shlex, subprocess, sys, tempfile

def aval_cmd():
    b = os.environ.get("AVAL_BIN")
    if b:
        return shlex.split(b)
    return ["cargo", "run", "-q", "-p", "aval-cli", "--"]

def run(args, cwd=None):
    return subprocess.run(aval_cmd() + args, capture_output=True, text=True, cwd=cwd)

def fail(msg):
    print("FAIL: " + msg)
    sys.exit(1)

src = "module t\nfn f() -> Int\n{\n  let y = 1\n  return y\n}\n"
with tempfile.TemporaryDirectory() as d:
    p = os.path.join(d, "main.aval")
    open(p, "w").write(src)
    r = run(["check", p])
lines = [json.loads(l) for l in r.stdout.splitlines() if l.strip()]
if r.returncode != 1 or len(lines) != 1 or lines[0].get("code") != "E002":
    fail("expected one E002 and exit 1, got %d %r" % (r.returncode, r.stdout))
if ":" not in lines[0].get("expected", []):
    fail("'expected' does not contain ':': %r" % (lines[0],))
print("ok")
