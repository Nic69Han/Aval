#!/usr/bin/env python3
# scenario: CLI-002
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

r = run(["--version"])
if r.returncode != 0:
    fail("exit code %d" % r.returncode)
if not r.stdout.startswith("aval 0.1."):
    fail("stdout does not start with 'aval 0.1.': %r" % r.stdout)
print("ok")
