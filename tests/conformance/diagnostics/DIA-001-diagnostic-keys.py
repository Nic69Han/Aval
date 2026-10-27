#!/usr/bin/env python3
# scenario: DIA-001
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

src = "module t\nfn f(x: Int) -> Int\n{\n  return y\n}\n"
with tempfile.TemporaryDirectory() as d:
    p = os.path.join(d, "main.aval")
    open(p, "w").write(src)
    r = run(["check", p])
lines = [l for l in r.stdout.splitlines() if l.strip()]
if len(lines) != 1:
    fail("expected exactly one stdout line, got %r" % r.stdout)
o = json.loads(lines[0])
if not isinstance(o, dict):
    fail("not a JSON object")
for k in ("code", "at", "msg", "fn"):
    if k not in o:
        fail("missing key %s in %s" % (k, lines[0]))
if not (isinstance(o["at"], list) and len(o["at"]) == 2 and all(isinstance(v, int) and v >= 1 for v in o["at"])):
    fail("bad 'at': %r" % (o["at"],))
if o["at"] != [4, 10] or o["fn"] != "f" or o["code"] != "E101":
    fail("unexpected content: %s" % lines[0])
print("ok")
