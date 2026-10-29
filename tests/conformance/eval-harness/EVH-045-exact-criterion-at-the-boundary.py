#!/usr/bin/env python3
# scenario: EVH-045
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _aggregate import *

N = ids_n(20)


def case(aval_ok, rust_ok):
    """aval_ok / rust_ok: first-attempt successes (of 20) in each of the three runs."""
    with workdir() as w:
        paths = []
        for i in range(3):
            a = solved_map(0, fail_ids=N[aval_ok[i]:], ids=N)
            b = solved_map(0, fail_ids=N[rust_ok[i]:], ids=N)
            paths.append(write_run(w, "r%d" % (i + 1), make_results(a, b, ids=N)))
        out = os.path.join(w, "out")
        r = aggregate(out, paths)
        check(r.returncode == 0, "matching runs rejected: " + (r.stdout + r.stderr)[-500:])
        md, js = load_out(out)
        check(js["verdict"]["mean"] == mean_line(md), "verdict.mean and the report line disagree")
        check(js["verdict"]["every_run"] is (every_line(md) == "yes"), "verdict.every_run and the report line disagree")
        return js["verdict"]["mean"], js["verdict"]["every_run"]


# 12/20 = 0.6 against 15/20 = 0.75: exactly 0.8 x, although 0.8 * 0.75 > 0.6 in floating point
check(case([12] * 3, [15] * 3) == ("PASS", True), "12/20 vs 15/20 is exactly on the boundary: both verdicts must be PASS")
# just below: 11/20 vs 15/20
check(case([11] * 3, [15] * 3) == ("FAIL", False), "11/20 vs 15/20 is below the boundary: both verdicts must be FAIL")
# mean exactly on the boundary (11, 12, 13 -> 12) while a run is below it
check(case([11, 12, 13], [15] * 3) == ("PASS", False), "mean 12/20 vs 15/20 holds, run 1 (11/20) does not")
# another exact boundary: 8/20 = 0.4 against 10/20 = 0.5
check(case([8] * 3, [10] * 3) == ("PASS", True), "8/20 vs 10/20 is exactly on the boundary")
print("ok")
