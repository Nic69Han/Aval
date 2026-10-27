#!/usr/bin/env python3
# scenario: EVH-003
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

# HE-002 answer that compiles but is wrong only for the hidden cases whose input is exactly 0.25
task = json.load(open(os.path.join(REPO, "experiments", "e2-in-context", "tasks", "HE-002.json")))
bad = [i for i, h in enumerate(task["hidden"]) if h["args"][0] == 0.25]
check(bad, "setup: HE-002 must contain a case with input 0.25")
with tmp() as out:
    run(["--provider", "replay", "--responses", fx("wrong_one_case.json"), "--task", "HE-002", "--langs", "aval", "--repairs", "0"], out)
    a = results(out)["tasks"]["HE-002"]["aval"]
    check(a["solved"] is False, "task must not be solved")
    check(a["pass_at_1"] is False, "pass_at_1 must be false")
    check(a["compile_at_1"] is True, "the answer compiles")
    check(len(a["attempts"]) == 1, "one attempt expected")
    check(a["attempts"][0]["failed_cases"] == bad, "failing case index: %r" % a["attempts"][0]["failed_cases"])
    check(a["failed_cases"] == bad, "task-level failed_cases: %r" % a["failed_cases"])
# twin: the reference answer passes the same case
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--task", "HE-002", "--langs", "aval"], out)
    a = results(out)["tasks"]["HE-002"]["aval"]
    check(a["solved"] is True and a["failed_cases"] == [], "reference must solve HE-002")
print("ok")
