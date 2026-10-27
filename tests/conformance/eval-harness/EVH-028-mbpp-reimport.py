#!/usr/bin/env python3
# scenario: EVH-028
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *
MBPP = os.path.join(REPO, "experiments", "e2-in-context", "tasks-mbpp")

IMPORTER = os.path.join(REPO, "experiments", "e2-in-context", "tools", "import_mbpp_plus.py")
try:
    import pyarrow  # noqa: F401
except ImportError:
    fail("PYARROW: this test needs pyarrow in the Python that runs it (pip install pyarrow==26.0.0)")
pq = fetch_mbpp_parquet()
with tmp() as d:
    out = os.path.join(d, "out")
    r = subprocess.run([sys.executable, "-I", IMPORTER, pq, "--out", out, "--excluded", os.path.join(MBPP, "EXCLUDED.md")],
                       capture_output=True, text=True)
    check(r.returncode == 0, "importer failed: %s %s" % (r.stdout[-500:], r.stderr[-500:]))
    mine = sorted(f for f in os.listdir(out) if f.endswith(".json"))
    theirs = sorted(f for f in os.listdir(MBPP) if f.endswith(".json"))
    check(mine == theirs and len(mine) >= 150, "file lists differ: %d regenerated vs %d shipped; only-regenerated %s, only-shipped %s" %
          (len(mine), len(theirs), sorted(set(mine) - set(theirs))[:5], sorted(set(theirs) - set(mine))[:5]))
    for f in mine:
        check(open(os.path.join(out, f), "rb").read() == open(os.path.join(MBPP, f), "rb").read(), f + ": regenerated file differs from the shipped one")
    # a modified file is refused and nothing is written
    bad = os.path.join(d, "modified.parquet")
    data = bytearray(open(pq, "rb").read())
    data[len(data) // 2] ^= 0x01
    open(bad, "wb").write(bytes(data))
    out2 = os.path.join(d, "out2")
    r = subprocess.run([sys.executable, "-I", IMPORTER, bad, "--out", out2], capture_output=True, text=True)
    check(r.returncode != 0, "a modified file must be refused")
    check("SHA-256" in r.stdout + r.stderr, "the refusal must mention the SHA-256: %r" % (r.stdout + r.stderr)[-300:])
    check(not os.path.exists(out2) or not [f for f in os.listdir(out2) if f.endswith(".json")], "no task file may be written for a refused input")
print("ok")
