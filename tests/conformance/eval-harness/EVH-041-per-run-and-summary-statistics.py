#!/usr/bin/env python3
# scenario: EVH-041
import os, statistics, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _aggregate import *

# Aval: run 1 fails MB-005, 007, 009, 010 (MB-010 is solved by a repair at attempt 2: still not a first-attempt
# success); run 2 fails MB-008, 009, 010 and has a provider error on MB-006; run 3 fails MB-010.
# Per-run pass@1 (metrics in each file, over 10 tasks): 0.6, 0.6, 0.9.
# Flips: MB-005, 007, 008, 009; MB-006 is excluded (provider error in a run), so 4 of 9 tasks.
# Rust: run 3 fails MB-001 -> pass@1 1.0, 1.0, 0.9; one flip of 10.
aval = [solved_map(0, fail_ids=f) for f in (["MB-005", "MB-007", "MB-009", "MB-010"], ["MB-008", "MB-009", "MB-010"], ["MB-010"])]
aval[0]["MB-010"] = 2
aval[1]["MB-006"] = "error"
rust = [solved_map(0, fail_ids=f) for f in ([], [], ["MB-001"])]
EXPECT = {"aval": ([0.6, 0.6, 0.9], 4 / 9), "rust": ([1.0, 1.0, 0.9], 0.1)}

with workdir() as w:
    paths = [write_run(w, "r%d" % (i + 1), make_results(aval[i], rust[i])) for i in range(3)]
    out = os.path.join(w, "out")
    r = aggregate(out, paths)
    check(r.returncode == 0, "three matching runs must be accepted, exit %d: %s" % (r.returncode, (r.stdout + r.stderr)[-500:]))
    md, js = load_out(out)
    check(js.get("runs") == 3, "aggregate.json runs must be 3, got %r" % js.get("runs"))
    mdn = numbers(md)
    for lang, (vals, flip) in EXPECT.items():
        L = js["languages"][lang]
        check(len(L["pass_at_1"]) == 3 and all(close(a, b) for a, b in zip(L["pass_at_1"], vals)),
              "%s pass_at_1 (in the order of the files) should be %r, got %r" % (lang, vals, L["pass_at_1"]))
        check(close(L["mean"], statistics.mean(vals)), "%s mean should be %r, got %r" % (lang, statistics.mean(vals), L["mean"]))
        check(close(L["stdev"], statistics.stdev(vals)), "%s stdev (sample, n-1) should be %r, got %r" % (lang, statistics.stdev(vals), L["stdev"]))
        check(close(L["flip_rate"], flip), "%s flip_rate should be %r, got %r" % (lang, flip, L["flip_rate"]))
        for what, x in [("pass@1 of a run", v) for v in vals] + [("mean", statistics.mean(vals)), ("stdev", statistics.stdev(vals)), ("flip rate", flip)]:
            check(has_value(mdn, x), "AGGREGATE.md (%s): %s %r not found as a fraction or a percentage:\n%s" % (lang, what, x, md))
    check(sum(1 for l in md.splitlines() if l.lstrip().startswith("|")) >= 5, "AGGREGATE.md must contain a table with a header and one row per run:\n" + md)
    check("aval" in md.lower() and "rust" in md.lower(), "report must be per language")

# every Aval task has a provider error in every run: no countable task, so the flip rate is n/a (null), not 0
with workdir() as w:
    allerr = {t: "error" for t in IDS}
    paths = [write_run(w, "r%d" % (i + 1), make_results(allerr, rust[i])) for i in range(3)]
    out = os.path.join(w, "out")
    r = aggregate(out, paths)
    check(r.returncode == 0, "runs accepted: " + (r.stdout + r.stderr)[-500:])
    md, js = load_out(out)
    check(js["languages"]["aval"]["flip_rate"] is None, "aval flip_rate with no countable task must be null, got %r" % js["languages"]["aval"]["flip_rate"])
    check(close(js["languages"]["rust"]["flip_rate"], 0.1), "rust flip_rate should still be 0.1")
    check("n/a" in md, "the table must show the flip rate as n/a (stdev is numeric here):\n" + md)
print("ok")
