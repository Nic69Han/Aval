#!/usr/bin/env python3
# scenario: EVH-020
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

with tmp() as out:
    run(["--provider", "replay", "--responses", fx("no_code_all.json"), "--task", "HE-002", "--task", "HE-003", "--task", "HE-013", "--repairs", "0"], out)
    md = open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read()
    check(re.search(r"^Verdict: INCONCLUSIVE", md, re.M), "RESULTS.md needs 'Verdict: INCONCLUSIVE'")
    res = results(out)
    check(res["verdict"]["result"] == "INCONCLUSIVE", "verdict %r" % res["verdict"]["result"])
    check(res["metrics"]["rust"]["pass_at_1"] == 0, "setup: Rust pass@1 must be 0")
# twin: Rust solves everything -> not inconclusive
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--task", "HE-002", "--task", "HE-003"], out)
    check(results(out)["verdict"]["result"] == "PASS", "healthy run must be PASS")
print("ok")
