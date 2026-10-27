#!/usr/bin/env python3
# scenario: TST-028
import json, os, shlex, subprocess, sys, tempfile

def aval_cmd():
    b = os.environ.get("AVAL_BIN")
    if b:
        return shlex.split(b)
    return ["cargo", "run", "-q", "-p", "aval-cli", "--"]

src = """module t
fn f(n: Int) -> Int
  examples
    1 => 0
{
  return f(n + 1)
}
"""
with tempfile.TemporaryDirectory() as d:
    p = os.path.join(d, "main.aval")
    open(p, "w").write(src)
    r = subprocess.run(aval_cmd() + ["test", p], capture_output=True, text=True, timeout=120)
if r.returncode != 1:
    print("FAIL: exit code %d" % r.returncode); sys.exit(1)
lines = [json.loads(l) for l in r.stdout.splitlines() if l.strip()]
if len(lines) != 1 or lines[0].get("code") != "T700" or lines[0].get("fn") != "f" or lines[0].get("fault") not in ("stack_limit", "step_limit"):
    print("FAIL: unexpected output %r" % r.stdout); sys.exit(1)
print("ok")
