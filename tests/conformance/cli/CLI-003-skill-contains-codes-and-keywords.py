#!/usr/bin/env python3
# scenario: CLI-003
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
KEYWORDS = ['fn', 'let', 'return', 'type', 'enum', 'module', 'import', 'pub', 'if', 'else', 'match', 'for', 'in', 'par', 'requires', 'ensures', 'result', 'examples', 'oracle', 'effects', 'alloc', 'cost', 'latency', 'reuse', 'none', 'resource', 'using', 'unsafe', 'true', 'false', 'and', 'or', 'not', 'Int', 'Float', 'Bool', 'String', 'Byte', 'List', 'Set', 'Map', 'Queue', 'Image', 'Tensor', 'Result', 'Arena']
CODES = ['E001', 'E002', 'E101', 'E102', 'E103', 'E105', 'E106', 'E204', 'E301', 'E302', 'E303', 'E310', 'E420', 'E421', 'E510', 'E511', 'E900', 'W210', 'W901', 'T100', 'T200', 'T600', 'T700']
r = run(["skill", "print"])
if r.returncode != 0:
    fail("exit code %d" % r.returncode)
out = r.stdout
missing = [c for c in CODES if c not in out]
if missing:
    fail("missing diagnostic codes: %s" % missing)
words = set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", out))
missing = [k for k in KEYWORDS if k not in words]
if missing:
    fail("missing keywords: %s" % missing)
print("ok")
