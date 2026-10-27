#!/usr/bin/env python3
# scenario: EVH-015
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

# A wrapper around the real aval makes `aval test` die (SIGSEGV, or exit 134 as an abort does).
real = aval_bin()
def wrapper(d, how):
    p = os.path.join(d, "aval-crashing")
    open(p, "w").write('#!/bin/sh\nif [ "$1" = test ]; then\n  %s\nfi\nexec %s "$@"\n' % (how, shlex.quote(real)))
    os.chmod(p, 0o755)
    return p
for how in ("kill -SEGV $$", "exit 134"):
    with tmp() as out, tmp() as d:
        run(["--provider", "replay", "--responses", "reference", "--task", "HE-003", "--task", "HE-013", "--langs", "aval",
             "--repairs", "0", "--aval-bin", wrapper(d, how)], out)
        check(os.path.exists(os.path.join(out, "results.json")) and os.path.exists(os.path.join(out, "RESULTS.md")), "%s: outputs not written" % how)
        tasks = results(out)["tasks"]
        for tid in ("HE-003", "HE-013"):     # the run continues past the first crash
            a = tasks[tid]["aval"]
            check(a["solved"] is False, "%s %s: must not be solved" % (how, tid))
            check(a["attempts"][0]["outcome"] == "failed" and a["attempts"][0]["reason"] == "crash",
                  "%s %s: reason %r" % (how, tid, a["attempts"][0]["reason"]))
# twin: the same wrapper-free run solves the task
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--task", "HE-003", "--langs", "aval"], out)
    check(results(out)["tasks"]["HE-003"]["aval"]["solved"] is True, "reference must be solved")
# a candidate that really exhausts memory still ends as a failed attempt
with tmp() as out:
    run(["--provider", "replay", "--responses", fx("aval_abort.json"), "--task", "HE-003", "--langs", "aval", "--repairs", "0"], out, timeout=600)
    a = results(out)["tasks"]["HE-003"]["aval"]
    check(a["solved"] is False and a["attempts"][0]["outcome"] == "failed", "memory-hungry candidate must fail: %r" % a["attempts"][0])
print("ok")
