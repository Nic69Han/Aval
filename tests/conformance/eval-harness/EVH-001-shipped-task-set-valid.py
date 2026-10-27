#!/usr/bin/env python3
# scenario: EVH-001
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

r = harness(["validate-tasks"])
check(r.returncode == 0, "validate-tasks exited %d: %s %s" % (r.returncode, r.stdout, r.stderr))
check(re.search(r"\b40\b", r.stdout) and "valid" in r.stdout, "expected '40 valid tasks', got %r" % r.stdout)
# independent look at the shipped files
tdir = os.path.join(REPO, "experiments", "e2-in-context", "tasks")
files = sorted(f for f in os.listdir(tdir) if f.startswith("HE-") and f.endswith(".json"))
check(len(files) == 40, "expected 40 task files, found %d" % len(files))
for f in files:
    t = json.load(open(os.path.join(tdir, f)))
    check(t["id"] + ".json" == f, f + ": id mismatch")
    check("HumanEval" in t["source"] and "MIT" in t["source"], f + ": source attribution")
    check(len(t["hidden"]) >= 5, f + ": fewer than 5 hidden cases")
    check(t["statement"].strip() and "```" not in t["statement"], f + ": statement")
    check(t["rust_signature"].startswith("fn " + t["aval"]["name"]), f + ": rust signature")
    for lang in ("aval", "rust"):
        check(os.path.exists(os.path.join(REPO, t["reference"][lang])), f + ": missing reference " + lang)
        body = open(os.path.join(REPO, t["reference"][lang])).read()
        check(body.count("```") == 2 and body.startswith("```" + lang), f + ": reference must be one fenced block")
check(os.path.exists(os.path.join(tdir, "ATTRIBUTION.md")), "ATTRIBUTION.md missing")
print("ok")
