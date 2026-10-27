#!/usr/bin/env python3
# scenario: CLI-005
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
r = run(["skill", "print"])
if r.returncode != 0:
    fail("aval skill print exited with %d" % r.returncode)
blocks = re.findall(r"^```aval[ \t]*\n(.*?)^```[ \t]*$", r.stdout, re.S | re.M)
if not blocks:
    fail("no ```aval fenced block found in the skill output")
with tempfile.TemporaryDirectory() as d:
    for i, b in enumerate(blocks):
        p = os.path.join(d, "block%d.aval" % i)
        open(p, "w").write(b)
        c = run(["check", p])
        errors = []
        for line in c.stdout.splitlines():
            try:
                o = json.loads(line)
            except ValueError:
                continue
            if str(o.get("code", "")).startswith(("E", "T")):
                errors.append(line)
        if c.returncode != 0 or errors:
            fail("block %d: exit %d, %s" % (i, c.returncode, errors or c.stdout + c.stderr))
print("ok: %d block(s)" % len(blocks))
