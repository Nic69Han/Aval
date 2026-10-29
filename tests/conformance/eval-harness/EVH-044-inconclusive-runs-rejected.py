#!/usr/bin/env python3
# scenario: EVH-044
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _aggregate import *

base = solved_map(0, fail_ids=["MB-010"])
with workdir() as w:
    p1 = write_run(w, "r1", make_results(base, base))
    p2 = write_run(w, "r2", make_results(base, base, result="INCONCLUSIVE"))
    p3 = write_run(w, "r3", make_results(base, base))
    out = os.path.join(w, "out")
    r = aggregate(out, [p1, p2, p3])
    text = r.stdout + r.stderr
    check(r.returncode != 0, "a run with verdict INCONCLUSIVE must be rejected (non-zero exit), got 0")
    check(p2 in text or os.path.join("r2", "results.json") in text, "the message must name the offending file %s, got: %r" % (p2, text[-500:]))
    check(p1 not in text.replace(p2, "") and p3 not in text.replace(p2, ""), "the message must name only the offending file, got: %r" % text[-500:])
    check(out_untouched(out), "a rejected invocation must write nothing to --out")
    # twin: the same three runs with a decided verdict are accepted
    out2 = os.path.join(w, "out2")
    r = aggregate(out2, [p1, write_run(w, "r4", make_results(base, base)), p3])
    check(r.returncode == 0, "matching decided runs must be accepted: " + (r.stdout + r.stderr)[-500:])
print("ok")
