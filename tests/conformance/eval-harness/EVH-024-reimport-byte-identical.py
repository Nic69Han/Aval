#!/usr/bin/env python3
# scenario: EVH-024
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

import ssl, urllib.request
REVISION = "d32357cf319e50e9c8d8dab5ea876c72b0fd321b"     # evalplus/humanevalplus, last modified 2024-05-01
SHA256 = "908377f1daf28dcb36846db73a5662b2e05a9907407c2696c89ad9d3b0b04492"
URL = "https://huggingface.co/datasets/evalplus/humanevalplus/resolve/%s/test.jsonl" % REVISION
ctx = ssl.create_default_context()
for cand in (os.environ.get("SSL_CERT_FILE"), os.environ.get("REQUESTS_CA_BUNDLE"), "/root/.ccr/ca-bundle.crt"):
    if cand and os.path.exists(cand):
        ctx = ssl.create_default_context(cafile=cand)
        break
IMPORTER = os.path.join(REPO, "experiments", "e2-in-context", "tools", "import_humaneval_plus.py")
SHIPPED = os.path.join(REPO, "experiments", "e2-in-context", "tasks")
with tmp() as d:
    jsonl = os.path.join(d, "test.jsonl")
    try:
        with urllib.request.urlopen(URL, timeout=300, context=ctx) as resp, open(jsonl, "wb") as f:
            f.write(resp.read())
    except Exception as e:
        fail("NETWORK: cannot download %s (this test needs network access and is not skipped): %r" % (URL, e))
    digest = hashlib.sha256(open(jsonl, "rb").read()).hexdigest()
    check(digest == SHA256, "DATASET CHANGED: SHA-256 of the downloaded test.jsonl is %s, expected %s (revision %s)" % (digest, SHA256, REVISION))
    out = os.path.join(d, "out")
    r = subprocess.run([sys.executable, "-I", IMPORTER, jsonl, "--tasks", SHIPPED, "--out", out], capture_output=True, text=True)
    check(r.returncode == 0, "importer failed: %s %s" % (r.stdout[-500:], r.stderr[-500:]))
    mine = sorted(f for f in os.listdir(out) if f.endswith(".json"))
    theirs = sorted(f for f in os.listdir(SHIPPED) if f.endswith(".json"))
    check(mine == theirs and len(mine) == 40, "file lists differ: %d vs %d" % (len(mine), len(theirs)))
    for f in mine:
        check(open(os.path.join(out, f), "rb").read() == open(os.path.join(SHIPPED, f), "rb").read(), f + ": regenerated file differs from the shipped one")
    # determinism: a second run from the regenerated files gives the same bytes
    out2 = os.path.join(d, "out2")
    r = subprocess.run([sys.executable, "-I", IMPORTER, jsonl, "--tasks", out, "--out", out2], capture_output=True, text=True)
    check(r.returncode == 0, "second import failed")
    for f in mine:
        check(open(os.path.join(out2, f), "rb").read() == open(os.path.join(SHIPPED, f), "rb").read(), f + ": second import differs")
print("ok")
