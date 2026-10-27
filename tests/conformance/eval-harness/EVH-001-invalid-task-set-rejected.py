#!/usr/bin/env python3
# scenario: EVH-001
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

import shutil
src = os.path.join(REPO, "experiments", "e2-in-context", "tasks")

def variant(mutate):
    d = tempfile.mkdtemp()
    dst = os.path.join(d, "tasks")
    shutil.copytree(src, dst)
    p = os.path.join(dst, "HE-003.json")
    t = json.load(open(p))
    mutate(t, dst)
    json.dump(t, open(p, "w"))
    return dst

# untouched copy: valid (guards against a validator that rejects everything)
r = harness(["validate-tasks", "--tasks", variant(lambda t, d: None)])
check(r.returncode == 0 and re.search(r"\b40\b", r.stdout), "untouched copy should be valid: %s %s" % (r.stdout, r.stderr))

def few_cases(t, d): t["hidden"] = t["hidden"][:4]
def no_statement(t, d): del t["statement"]
def no_source(t, d): del t["source"]
def bad_reference(t, d): t["reference"]["rust"] = "experiments/e2-in-context/reference/does-not-exist.rs"
def no_signature(t, d): del t["rust_signature"]
for name, m in [("four hidden cases", few_cases), ("no statement", no_statement), ("no source", no_source),
                ("missing reference", bad_reference), ("no rust signature", no_signature)]:
    r = harness(["validate-tasks", "--tasks", variant(m)])
    check(r.returncode == 1, "%s: expected exit 1, got %d" % (name, r.returncode))
    check("HE-003" in r.stdout + r.stderr, "%s: the invalid task id is not reported" % name)
    check(not re.search(r"\b40 valid", r.stdout), "%s: must not report 40 valid tasks" % name)
print("ok")
