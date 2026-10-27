#!/usr/bin/env python3
# scenario: CLI-004
import os, shlex, subprocess, sys

def aval_cmd():
    b = os.environ.get("AVAL_BIN")
    if b:
        return shlex.split(b)
    return ["cargo", "run", "-q", "-p", "aval-cli", "--"]

try:
    import tiktoken
except ImportError:
    print("FAIL: tiktoken is not installed (pip install tiktoken)")
    sys.exit(1)

r = subprocess.run(aval_cmd() + ["skill", "print"], capture_output=True, text=True)
if r.returncode != 0:
    print("FAIL: aval skill print exited with %d" % r.returncode)
    sys.exit(1)
n = len(tiktoken.get_encoding("o200k_base").encode(r.stdout))
if n > 4000:
    print("FAIL: skill is %d tokens (budget 4000)" % n)
    sys.exit(1)
print("ok: %d tokens" % n)
