#!/usr/bin/env python3
# scenario: TST-019
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

import time
src = """module t
fn fib(n: Int) -> Int
  requires n >= 0
  ensures result >= 0
{
  return if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}
"""
with tempfile.TemporaryDirectory() as d:
    p = os.path.join(d, "main.aval")
    open(p, "w").write(src)
    t0 = time.time()
    try:
        r = subprocess.run(aval_cmd() + ["test", p], capture_output=True, text=True, timeout=120)
    except subprocess.TimeoutExpired:
        fail("aval test did not finish within 120 s")
    dt = time.time() - t0
if dt >= 60:
    fail("took %.1f s (limit 60 s)" % dt)
lines = [json.loads(l) for l in r.stdout.splitlines() if l.strip()]
if r.returncode != 1:
    fail("expected exit 1, got %d" % r.returncode)
if not any(o.get("code") == "T700" and o.get("fault") == "step_limit" and o.get("fn") == "fib" for o in lines):
    fail("no T700 step_limit for fib in %r" % r.stdout)
print("ok: %.1f s" % dt)
