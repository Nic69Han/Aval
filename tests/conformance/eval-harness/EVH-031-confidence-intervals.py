#!/usr/bin/env python3
# scenario: EVH-031
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *
MBPP = os.path.join(REPO, "experiments", "e2-in-context", "tasks-mbpp")

IDS = ["MB-008", "MB-017", "MB-018", "MB-059", "MB-079", "MB-089", "MB-135", "MB-227"]   # the Aval answers of 018, 079 and 227 have no code
def ci_lines(out):
    md = open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read()
    lines = [l.strip().lstrip("-* ").strip() for l in md.splitlines() if "95% CI" in l]
    return lines
def parse(line):
    m = re.search(r"\[\s*(-?[0-9.]+(?:e-?\d+)?)\s*,\s*(-?[0-9.]+(?:e-?\d+)?)\s*\]", line)
    check(m, "unparsable interval in %r" % line)
    return float(m.group(1)), float(m.group(2))
def once():
    with tmp() as out:
        run(["--provider", "replay", "--responses", fx("mbpp_some_fail.json"), "--tasks", MBPP, "--repairs", "0"] +
            [a for t in IDS for a in ("--task", t)], out)
        return ci_lines(out), results(out)
a, res = once()
b, _ = once()
check(a == b, "interval lines differ between two identical runs:\n%s\n---\n%s" % (a, b))
aval = [l for l in a if re.match(r"Aval\b.*pass@1 95% CI", l)]
rust = [l for l in a if re.match(r"Rust\b.*pass@1 95% CI", l)]
diff = [l for l in a if l.startswith("Aval") and "Rust 95% CI" in l]
check(len(aval) == 1 and len(rust) == 1 and len(diff) == 1, "expected one line per interval, got %r" % a)
lo, hi = parse(aval[0])
p = res["metrics"]["aval"]["pass_at_1"]
check(abs(p - 5 / 8) < 1e-9, "setup: Aval pass@1 should be 0.625, got %r" % p)
scale = 100.0 if hi > 1.0 else 1.0
check(lo / scale < p < hi / scale + 1e-12 and lo < hi, "Aval interval [%s, %s] must bracket pass@1 = %s" % (lo, hi, p))
rlo, rhi = parse(rust[0])
check(rlo == rhi == scale * 1.0, "Rust solved everything: interval must be degenerate at 1, got [%s, %s]" % (rlo, rhi))
dlo, dhi = parse(diff[0])
check(dlo < 0 and dhi <= 1e-9 and dlo < dhi, "the paired difference must be negative, got [%s, %s]" % (dlo, dhi))
print("ok")
