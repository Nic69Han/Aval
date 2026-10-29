#!/usr/bin/env python3
# scenario: EVH-043
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _aggregate import *

fails = [["MB-009", "MB-010"], ["MB-010"]]
for n in (1, 2):
    with workdir() as w:
        paths = [write_run(w, "r%d" % (i + 1), make_results(solved_map(0, fail_ids=fails[i]), solved_map(0))) for i in range(n)]
        out = os.path.join(w, "out")
        r = aggregate(out, paths)
        check(r.returncode == 0, "%d run(s): the report must still be produced, exit %d: %s" % (n, r.returncode, (r.stdout + r.stderr)[-500:]))
        md, js = load_out(out)
        check(js["runs"] == n, "runs should be %d" % n)
        for lang in ("aval", "rust"):
            check(js["languages"][lang]["stdev"] is None, "%d run(s): %s stdev must be null, got %r" % (n, lang, js["languages"][lang]["stdev"]))
            check(len(js["languages"][lang]["pass_at_1"]) == n, "per-run pass_at_1 must have %d entries" % n)
        want_mean = sum([0.8, 0.9][:n]) / n
        check(close(js["languages"]["aval"]["mean"], want_mean), "aval mean should be %r, got %r" % (want_mean, js["languages"]["aval"]["mean"]))
        check("n/a" in md, "%d run(s): the table must show the deviation as n/a:\n%s" % (n, md))
        warn = (r.stdout + r.stderr + md).lower()
        check("not meaningful" in warn, "%d run(s): a warning that the deviation is not meaningful is required (stderr, stdout or report)" % n)

    # no countable task for Aval: flip rate null
    with workdir() as w:
        allerr = {t: "error" for t in IDS}
        paths = [write_run(w, "r%d" % (i + 1), make_results(allerr, solved_map(0))) for i in range(n)]
        out = os.path.join(w, "out")
        r = aggregate(out, paths)
        check(r.returncode == 0, "%d run(s) accepted: %s" % (n, (r.stdout + r.stderr)[-500:]))
        md, js = load_out(out)
        check(js["languages"]["aval"]["flip_rate"] is None, "%d run(s): flip_rate with no countable task must be null" % n)
        check(js["languages"]["rust"]["flip_rate"] == 0, "%d run(s): rust flip_rate should be 0" % n)
print("ok")
