#!/usr/bin/env python3
# scenario: EVH-021
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

import shutil
r = harness(["validate-tasks"])
check(r.returncode == 0 and re.search(r"\b40\b", r.stdout), "validate-tasks: exit %d %s %s" % (r.returncode, r.stdout, r.stderr))
src = os.path.join(REPO, "experiments", "e2-in-context", "tasks")
files = sorted(f for f in os.listdir(src) if re.fullmatch(r"HE-\d+\.json", f))
check(len(files) == 40, "expected 40 task files")
total = 0
for f in files:
    t = json.load(open(os.path.join(src, f)))
    check("HumanEval+" in t.get("source_tests", ""), f + ": source_tests must mention HumanEval+")
    check("EvalPlus" in t["source_tests"] and "Apache" in t["source_tests"], f + ": source_tests must name EvalPlus and its licence")
    check(isinstance(t.get("excluded"), dict), f + ": excluded must be an object")
    check(all(isinstance(v, int) and v > 0 for v in t["excluded"].values()), f + ": excluded counts")
    check(len(t["hidden"]) >= 20, f + ": only %d hidden cases" % len(t["hidden"]))
    check(isinstance(t.get("atol"), (int, float)) and t["atol"] >= 0, f + ": atol")
    if t["aval"]["returns"] in ("Float", "List<Float>"):
        check(t["atol"] > 0, f + ": Float tasks need a positive atol")
    total += len(t["hidden"])
check(total > 20000, "suspiciously few hidden cases in total: %d" % total)
# a task file that loses its provenance is rejected
d = tempfile.mkdtemp()
def variant(mutate):
    dst = os.path.join(d, "t%d" % len(os.listdir(d)))
    shutil.copytree(src, dst)
    p = os.path.join(dst, "HE-003.json")
    t = json.load(open(p)); mutate(t); json.dump(t, open(p, "w"))
    return dst
def del_key(k):
    def m(t): del t[k]
    return m
def plain_source(t): t["source_tests"] = "HumanEval (MIT)"
def few(t): t["hidden"] = t["hidden"][:19]
for name, m in [("no source_tests", del_key("source_tests")), ("source_tests without HumanEval+", plain_source),
                ("no excluded", del_key("excluded")), ("19 hidden cases", few)]:
    r = harness(["validate-tasks", "--tasks", variant(m)])
    check(r.returncode == 1, "%s: expected exit 1, got %d" % (name, r.returncode))
    check("HE-003" in r.stdout + r.stderr, "%s: invalid id not reported" % name)
print("ok")
