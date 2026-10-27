#!/usr/bin/env python3
# scenario: EVH-026
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

# The answer is correct except that, for the input 10, it burns far more steps than one batch is allowed.
task = json.load(open(os.path.join(REPO, "experiments", "e2-in-context", "tasks", "HE-046.json")))
slow = [i for i, h in enumerate(task["hidden"]) if h["args"][0] == 10]
check(slow, "setup: HE-046 must contain the input 10")
with tmp() as out:
    run(["--provider", "replay", "--responses", fx("slow_on_one_case.json"), "--task", "HE-046", "--langs", "aval", "--repairs", "0",
         "--timeout", "400"], out, timeout=1800)
    a = results(out)["tasks"]["HE-046"]["aval"]
    att = a["attempts"][0]
    check(a["solved"] is False and att["failed_cases"] == slow, "only the slow case may fail: %r" % att["failed_cases"][:5])
    kinds = att.get("fault_kinds")
    check(kinds is not None and "step_limit" in kinds, "fault_kinds must contain step_limit: %r" % (kinds,))
    md = open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read()
    rows = [l for l in md.splitlines() if "resource failures" in l.lower()]
    check(rows, "RESULTS.md must contain a 'resource failures' row")
    check(any(re.search(r"\b1\b", l) for l in rows), "the row must count the failed task: %r" % rows)
# twin: the reference has no fault kinds and the row counts 0
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--task", "HE-046", "--langs", "aval"], out)
    att = results(out)["tasks"]["HE-046"]["aval"]["attempts"][0]
    check(not att.get("fault_kinds"), "a correct answer records no fault kinds: %r" % att.get("fault_kinds"))
print("ok")
