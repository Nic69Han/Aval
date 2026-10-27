#!/usr/bin/env python3
# scenario: EVH-012
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

def once():
    with tmp() as out:
        run(["--provider", "replay", "--responses", fx("verdict_borderline_pass.json"), "--repairs", "2",
             "--task", "HE-002", "--task", "HE-003", "--task", "HE-013", "--task", "HE-015", "--task", "HE-048"], out)
        res = results(out)
        res.pop("generated_at", None)
        md = open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read()
        md = "\n".join(l for l in md.splitlines() if not re.search(r"generated|timestamp|\b\d{4}-\d{2}-\d{2}", l, re.I))
        return json.dumps(res, sort_keys=True), md
a, b = once(), once()
check(a[0] == b[0], "results.json differs between two identical replay runs")
check(a[1] == b[1], "RESULTS.md differs between two identical replay runs")
check("Verdict:" in a[1], "RESULTS.md has no verdict")
check(len(a[0]) > 1000, "results.json suspiciously small")
print("ok")
