#!/usr/bin/env python3
# scenario: TYP-027
import json, os, shlex, subprocess, sys, tempfile

def aval_cmd():
    b = os.environ.get("AVAL_BIN")
    return shlex.split(b) if b else ["cargo", "run", "-q", "-p", "aval-cli", "--"]

def check(src):
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "main.aval")
        open(p, "w").write(src)
        return subprocess.run(aval_cmd() + ["check", p], capture_output=True, text=True)

def fail(m):
    print("FAIL: " + m)
    sys.exit(1)

bad = """module t
fn f(xs: List<Int>) -> List<Int>
{
  return xs.fold([], |acc, x| acc.map(|y| y.len()).push(x))
}
"""
r = check(bad)
codes = [json.loads(l).get("code") for l in r.stdout.splitlines() if l.strip()]
if r.returncode != 1:
    fail("expected exit 1, got %d" % r.returncode)
if len(codes) != 1 or codes[0] not in ("E204", "E301"):
    fail("expected exactly one E204 or E301, got %r" % codes)
good = bad.replace("acc.map(|y| y.len()).push(x)", "acc.push(x)")
r = check(good)
if r.returncode != 0 or r.stdout.strip():
    fail("twin should be clean: %r" % r.stdout)
print("ok")
