#!/usr/bin/env python3
# scenario: EVH-025
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

# HE-046 has 64 hidden cases (batches of 50 and 14). The answer burns about 2.8 million steps per case
# (a 500,000-element list folded on every call): 50 such cases share far more than one per-input budget
# (10 million steps), but each case alone is well under it, so the task must be solved.
task = json.load(open(os.path.join(REPO, "experiments", "e2-in-context", "tasks", "HE-046.json")))
check(len(task["hidden"]) > 50, "setup: HE-046 needs more than one batch")
with tmp() as out:
    run(["--provider", "replay", "--responses", fx("slow_3m_steps.json"), "--task", "HE-046", "--langs", "aval", "--repairs", "0",
         "--timeout", "300"], out, timeout=1200)
    a = results(out)["tasks"]["HE-046"]["aval"]
    check(a["solved"] is True and a["failed_cases"] == [], "slow but correct answer must be solved: %r %r" % (a.get("reason"), a["failed_cases"][:5]))
print("ok")
