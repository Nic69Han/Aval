#!/usr/bin/env python3
# scenario: EVH-030
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *
MBPP = os.path.join(REPO, "experiments", "e2-in-context", "tasks-mbpp")

ids = ["MB-008", "MB-017", "MB-059", "MB-089", "MB-135"]
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--tasks", MBPP] + [a for t in ids for a in ("--task", t)], out)
    res = results(out)
    check(res.get("task_set") == "MBPP+", "task_set = %r" % res.get("task_set"))
    md = open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read()
    check(re.search(r"^Task set: MBPP\+\s*$", md, re.M), "RESULTS.md must contain the line 'Task set: MBPP+'")
    check(res["task_count"] == 5, "task_count %r" % res["task_count"])
# the HumanEval+ set names itself too (guards against a hard-coded name)
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--task", "HE-002", "--task", "HE-003"], out)
    res = results(out)
    md = open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read()
    check(res.get("task_set") == "HumanEval+" and re.search(r"^Task set: HumanEval\+\s*$", md, re.M), "HumanEval+ run: %r" % res.get("task_set"))
print("ok")
