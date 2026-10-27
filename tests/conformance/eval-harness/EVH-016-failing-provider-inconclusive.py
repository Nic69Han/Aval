#!/usr/bin/env python3
# scenario: EVH-016
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

refdir = os.path.join(REPO, "experiments", "e2-in-context", "reference")
ANS = 'e=aval; [ "$AVAL_LANG" = rust ] && e=rs; cat %s/$AVAL_TASK_ID.$e' % shlex.quote(refdir)
cmd = 'if [ "$AVAL_TASK_ID" = HE-003 ]; then exit 1; else %s; fi' % ANS
with tmp() as out:
    run(["--provider", "command", "--command", cmd, "--task", "HE-002", "--task", "HE-003"], out)
    res = results(out)
    for lang in ("aval", "rust"):
        bad = res["tasks"]["HE-003"][lang]
        check(bad["outcome"] == "provider_error", "HE-003 %s outcome %r" % (lang, bad["outcome"]))
        check(res["tasks"]["HE-002"][lang]["solved"] is True, "HE-002 %s must be solved" % lang)
        check(res["metrics"][lang]["pass_at_1"] == 1.0, "the provider_error task must be excluded from the metrics: %r" % res["metrics"][lang]["pass_at_1"])
    check(res["verdict"]["result"] == "INCONCLUSIVE", "verdict %r" % res["verdict"]["result"])
    md = open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read()
    check(re.search(r"^Verdict: INCONCLUSIVE", md, re.M), "RESULTS.md needs a 'Verdict: INCONCLUSIVE' line")
# twin: with no failing task the verdict is not INCONCLUSIVE
with tmp() as out:
    run(["--provider", "command", "--command", ANS, "--task", "HE-002", "--task", "HE-003"], out)
    md = open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read()
    check("INCONCLUSIVE" not in md.split("Verdict:")[1].splitlines()[0], "healthy run must not be inconclusive")
print("ok")
