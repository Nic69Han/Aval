#!/usr/bin/env python3
# scenario: EVH-042
import os, re, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _aggregate import *

# Rust solves everything at attempt 1 in every run, so the threshold is 0.8 in each run and for the mean.
rust = solved_map(0)


def case(aval_fail_counts, rust_maps=None):
    """aval_fail_counts: number of Aval first-attempt failures in each of the three runs (of 10 tasks)."""
    with workdir() as w:
        paths = []
        for i, k in enumerate(aval_fail_counts):
            rm = rust_maps[i] if rust_maps else rust
            paths.append(write_run(w, "r%d" % (i + 1), make_results(solved_map(0, fail_ids=IDS[:k]), rm)))
        out = os.path.join(w, "out")
        r = aggregate(out, paths)
        check(r.returncode == 0, "matching runs rejected: " + (r.stdout + r.stderr)[-500:])
        md, js = load_out(out)
        v = js["verdict"]
        check(v["mean"] == mean_line(md).strip(), "verdict.mean %r disagrees with the report line %r" % (v["mean"], mean_line(md)))
        check(v["every_run"] is (every_line(md) == "yes"), "verdict.every_run %r disagrees with the report line %r" % (v["every_run"], every_line(md)))
        triv = re.findall(r"^Rust pass@1 is 0: criterion holds trivially\s*$", md, re.M)
        return v["mean"], v["every_run"], len(triv)


# pass@1 0.7, 0.9, 0.9: mean 0.8333 >= 0.8 holds, run 1 (0.7) does not
check(case([3, 1, 1]) == ("PASS", False, 0), "mean holds but one run violates: expected (PASS, false)")
check(case([1, 1, 1]) == ("PASS", True, 0), "all runs satisfy: expected (PASS, true)")
# pass@1 0.7, 0.7, 0.9: mean 0.7667 < 0.8 fails although run 3 satisfies it
check(case([3, 3, 1]) == ("FAIL", False, 0), "mean fails: expected (FAIL, false)")
# pass@1 exactly 0.8 in every run (boundary: criterion is >=)
check(case([2, 2, 2]) == ("PASS", True, 0), "pass@1 = 0.8 = threshold: criterion >= holds")
# mean Rust pass@1 is 0: the criterion holds trivially
allfail = solved_map(0, fail_ids=IDS)
check(case([5, 5, 5], [allfail] * 3) == ("PASS", True, 1),
      "Rust pass@1 = 0: verdict PASS/true and exactly one line 'Rust pass@1 is 0: criterion holds trivially'")
print("ok")
