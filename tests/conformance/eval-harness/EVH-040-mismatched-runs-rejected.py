#!/usr/bin/env python3
# scenario: EVH-040
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _aggregate import *

base = solved_map(0, fail_ids=["MB-010"])


def base_run(**kw):
    return make_results(base, base, **kw)


def attempt(label, variant, word, expect_ok=False):
    with workdir() as w:
        out = os.path.join(w, "out")
        paths = [write_run(w, "r1", base_run()), write_run(w, "r2", base_run()), write_run(w, "r3", variant)]
        r = aggregate(out, paths)
        text = (r.stdout + r.stderr).lower()
        if expect_ok:
            check(r.returncode == 0, "%s: matching runs must be accepted, exit %d: %s" % (label, r.returncode, text[-500:]))
            return
        check(r.returncode != 0, "%s: mismatched runs must be rejected (exit status non-zero), got 0" % label)
        check(word in text, "%s: the message must name the differing field (%r), got: %r" % (label, word, text[-500:]))
        check(out_untouched(out), "%s: a rejected invocation must write nothing to --out" % label)


# negative twin: runs that differ only in outcomes / timestamps / run-to-run noise are matching runs
noisy = make_results(solved_map(0, fail_ids=["MB-009"]), base, generated_at="2026-10-30T00:00:00Z")
attempt("outcome noise", noisy, None, expect_ok=True)

other_ids = IDS[:-1] + ["MB-999"]
attempt("task ids", make_results({t: 1 for t in other_ids}, {t: 1 for t in other_ids}, ids=other_ids), "task")
fewer = IDS[:-1]
attempt("task count", make_results({t: 1 for t in fewer}, {t: 1 for t in fewer}, ids=fewer), "task")
attempt("skill hash", base_run(skill=OTHER_SKILL), "skill")
attempt("repair count", base_run(repairs=0), "repairs")
attempt("model", base_run(model="m-2"), "model")
attempt("model null vs recorded", base_run(model=None), "model")
attempt("provider", base_run(provider="replay"), "provider")
print("ok")
