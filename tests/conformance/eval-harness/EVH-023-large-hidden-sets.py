#!/usr/bin/env python3
# scenario: EVH-023
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

import time
src = os.path.join(REPO, "experiments", "e2-in-context", "tasks")
sizes = {f[:-5]: len(json.load(open(os.path.join(src, f)))["hidden"]) for f in os.listdir(src) if f.endswith(".json")}
check(sum(sizes.values()) > 20000 and max(sizes.values()) > 900, "setup: the shipped task set must have large hidden sets")
with tmp() as out:
    t0 = time.time()
    run(["--provider", "replay", "--responses", "reference"], out, timeout=3000)
    res = results(out)
    n = 0
    for tid, arms in res["tasks"].items():
        for lang, arm in arms.items():
            n += 1
            check(arm["solved"] is True, "%s %s not solved (reason %r)" % (tid, lang, arm.get("reason")))
            for att in arm["attempts"]:
                check(att.get("reason") != "timeout", "%s %s: timeout" % (tid, lang))
                check(att.get("hidden_total") == sizes[tid], "%s %s: hidden_total %r != %d" % (tid, lang, att.get("hidden_total"), sizes[tid]))
    check(n == 80, "expected 80 arms, got %d" % n)
    print("ok (%.0f s)" % (time.time() - t0))
