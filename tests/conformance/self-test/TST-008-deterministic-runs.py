#!/usr/bin/env python3
# scenario: TST-008
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

src = """module t
fn f(x: Int) -> Int
  ensures result < 100
{
  return x
}
fn g(xs: List<Float>) -> Float
  requires xs.len() > 0
  ensures approx(result * xs.len().float(), xs.sum())
{
  return xs.sum() / 2.0
}
fn h(xs: List<Int>) -> Int
  ensures result != 7
{
  return xs.len()
}
"""
with tempfile.TemporaryDirectory() as d:
    p = os.path.join(d, "main.aval")
    open(p, "w").write(src)
    a = run(["test", p])
    b = run(["test", p])
    c = run(["test", p, "--seed", "0"])
if a.returncode != 1 or b.returncode != 1:
    fail("expected exit 1, got %d / %d" % (a.returncode, b.returncode))
if a.stdout != b.stdout:
    fail("two runs differ:\n%s\n---\n%s" % (a.stdout, b.stdout))
if len([l for l in a.stdout.splitlines() if l.strip()]) != 3:
    fail("expected three T200 lines, got %r" % a.stdout)
if a.stdout != c.stdout:
    fail("default seed differs from --seed 0")
print("ok")
