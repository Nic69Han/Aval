#!/usr/bin/env python3
# scenario: EVH-002
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

# The full 40-task replay is EVH-023; this scenario runs a representative subset of 5 tasks.
SUBSET = ["HE-002", "HE-003", "HE-013", "HE-015", "HE-048"]
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference"] + [a for t in SUBSET for a in ("--task", t)], out)
    res = results(out)
    check(res["task_count"] == 5, "task_count %r" % res["task_count"])
    for lang in ("aval", "rust"):
        m = res["metrics"][lang]
        check(m["pass_at_1"] == 1.0, "%s pass@1 = %r" % (lang, m["pass_at_1"]))
        check(m["tasks"] == 5 and m["solved"] == 5, "%s: %r solved" % (lang, m["solved"]))
    for tid in SUBSET:
        for lang in ("aval", "rust"):
            a = res["tasks"][tid][lang]
            check(a["solved"] and a["pass_at_1"] and len(a["attempts"]) == 1, "%s %s not solved at attempt 1" % (tid, lang))
print("ok")
