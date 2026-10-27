#!/usr/bin/env python3
# scenario: STD-023
import json, os, shlex, subprocess, sys, tempfile

def aval_cmd():
    b = os.environ.get("AVAL_BIN")
    return shlex.split(b) if b else ["cargo", "run", "-q", "-p", "aval-cli", "--"]

def fail(m):
    print("FAIL: " + m)
    sys.exit(1)

src = """module t
fn f(n: Int) -> Int
{
  return (0..n).average()
}
"""
with tempfile.TemporaryDirectory() as d:
    p = os.path.join(d, "main.aval")
    open(p, "w").write(src)
    r = subprocess.run(aval_cmd() + ["check", p], capture_output=True, text=True)
if r.returncode != 1:
    fail("expected exit 1, got %d" % r.returncode)
lines = [json.loads(l) for l in r.stdout.splitlines() if l.strip()]
if len(lines) != 1 or lines[0].get("code") != "E204":
    fail("expected exactly one E204, got %r" % lines)
if lines[0].get("on") != "Range<Int>":
    fail("on = %r" % lines[0].get("on"))
avail = lines[0].get("avail")
want = ["len", "is_empty", "sum", "product", "min", "max", "contains", "count", "map", "filter",
        "fold", "any", "all", "reverse", "take", "skip"]
if not isinstance(avail, list):
    fail("avail is not a list: %r" % avail)
missing = [m for m in want if m not in avail]
if missing:
    fail("avail lacks %r (got %r)" % (missing, avail))
print("ok")
