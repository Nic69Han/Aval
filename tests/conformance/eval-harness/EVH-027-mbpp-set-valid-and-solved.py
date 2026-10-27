#!/usr/bin/env python3
# scenario: EVH-027
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *
MBPP = os.path.join(REPO, "experiments", "e2-in-context", "tasks-mbpp")

import time
r = harness(["validate-tasks", "--tasks", MBPP])
check(r.returncode == 0, "validate-tasks on tasks-mbpp: exit %d\n%s\n%s" % (r.returncode, r.stdout[-800:], r.stderr[-800:]))
m = re.search(r"\b(\d+)\b", r.stdout)
check(m and int(m.group(1)) >= 150, "expected at least 150 valid tasks, got %r" % r.stdout)
files = sorted(f for f in os.listdir(MBPP) if re.fullmatch(r"MB-\d+\.json", f))
check(len(files) >= 150, "only %d task files" % len(files))
for f in files:
    t = json.load(open(os.path.join(MBPP, f)))
    check("MBPP+" in t["source_tests"], f + ": source_tests must mention MBPP+")
    check(t["id"] + ".json" == f, f + ": id mismatch")
    for lang in ("aval", "rust"):
        check(os.path.exists(os.path.join(REPO, t["reference"][lang])), f + ": missing " + lang + " reference")
check(os.path.exists(os.path.join(MBPP, "EXCLUDED.md")), "EXCLUDED.md is missing")
# every task listed in EXCLUDED.md has no task file
excl = re.findall(r"^\|\s*(MB-\d+)\s*\|", open(os.path.join(MBPP, "EXCLUDED.md"), encoding="utf-8").read(), re.M)
check(not [e for e in excl if os.path.exists(os.path.join(MBPP, e + ".json"))], "a task listed in EXCLUDED.md is also shipped")
with tmp() as out:
    t0 = time.time()
    run(["--provider", "replay", "--responses", "reference", "--tasks", MBPP], out, timeout=7200)
    res = results(out)
    check(res["task_set"] == "MBPP+", "task_set %r" % res.get("task_set"))
    for lang in ("aval", "rust"):
        m = res["metrics"][lang]
        check(m["tasks"] == len(files) and m["pass_at_1"] == 1.0, "%s: pass@1 = %r over %r tasks" % (lang, m["pass_at_1"], m["tasks"]))
    print("ok (%d tasks, %.0f s)" % (len(files), time.time() - t0))
