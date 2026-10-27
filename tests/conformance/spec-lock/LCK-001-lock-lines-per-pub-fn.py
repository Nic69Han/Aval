#!/usr/bin/env python3
# scenario: LCK-001
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

import re
src = """module m
pub fn b(x: Int) -> Int
  ensures result == x
  cost O(1)
{
  return x
}
fn c(x: Int) -> Int
{
  return x
}
pub fn a(x: Int) -> Int
  ensures result > x
  cost O(1)
{
  return x + 1
}
"""
with tempfile.TemporaryDirectory() as d:
    p = os.path.join(d, "main.aval")
    open(p, "w").write(src)
    r = run(["spec", "lock", p])
    if r.returncode != 0:
        fail("spec lock exited with %d: %s" % (r.returncode, r.stdout + r.stderr))
    lp = os.path.join(d, "aval.lock")
    if not os.path.exists(lp):
        fail("aval.lock not written next to the source file")
    lines = [l for l in open(lp).read().splitlines() if l.strip()]
    if len(lines) != 2:
        fail("expected 2 lines, got %r" % lines)
    pat = re.compile(r"^main\.aval:(m\.[ab]) sha256:[0-9a-f]{64}$")
    names = []
    for l in lines:
        m = pat.match(l)
        if not m:
            fail("bad lock line: %r" % l)
        names.append(m.group(1))
    if names != ["m.a", "m.b"]:
        fail("lines not exactly m.a then m.b: %r" % names)
    c = run(["check", p])
    if c.returncode != 0 or c.stdout.strip():
        fail("check after lock is not clean: %r" % c.stdout)
print("ok")
