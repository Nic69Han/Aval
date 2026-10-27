#!/usr/bin/env python3
# scenario: EVH-009
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

TASKS = ["--task", "HE-002", "--task", "HE-003", "--task", "HE-013", "--task", "HE-015", "--task", "HE-048"]
def verdict(resp):
    with tmp() as out:
        run(["--provider", "replay", "--responses", fx(resp), "--repairs", "0"] + TASKS, out)
        lines = [l for l in open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read().splitlines() if l.startswith("Verdict:")]
        check(len(lines) == 1, "expected one Verdict: line, got %r" % lines)
        m = re.match(r"Verdict:\s*(PASS|FAIL)\b.*?Aval pass@1\s*=\s*([0-9.]+).*?Rust pass@1\s*=\s*([0-9.]+)", lines[0])
        check(m, "unparsable verdict line %r" % lines[0])
        check(results(out)["verdict"]["result"] == m.group(1), "results.json verdict differs from RESULTS.md")
        return m.group(1), float(m.group(2)), float(m.group(3))
v, a, r = verdict("verdict_all_correct.json")
check((v, a, r) == ("PASS", 1.0, 1.0), "all correct: %r" % ((v, a, r),))
v, a, r = verdict("verdict_borderline_pass.json")   # 0.8 >= 0.8 * 1.0
check(v == "PASS" and abs(a - 0.8) < 1e-9 and r == 1.0, "borderline: %r" % ((v, a, r),))
v, a, r = verdict("verdict_fail.json")              # 0.6 < 0.8
check(v == "FAIL" and abs(a - 0.6) < 1e-9 and r == 1.0, "failing: %r" % ((v, a, r),))
print("ok")
