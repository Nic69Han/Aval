#!/usr/bin/env python3
# scenario: EVH-022
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

import shutil
def solved(resp, lang, tasks=None):
    with tmp() as out:
        extra = ["--tasks", tasks] if tasks else []
        run(["--provider", "replay", "--responses", fx(resp), "--task", "HE-002", "--langs", lang, "--repairs", "0"] + extra, out)
        return results(out)["tasks"]["HE-002"][lang]
t = json.load(open(os.path.join(REPO, "experiments", "e2-in-context", "tasks", "HE-002.json")))
check(t["atol"] == 1e-6, "setup: HE-002 atol is %r" % t["atol"])
for lang in ("aval", "rust"):
    a = solved("tolerance_1e-8.json", lang)
    check(a["solved"] is True and a["failed_cases"] == [], "%s: off by 1e-8 must pass with atol 1e-6: %r" % (lang, a["failed_cases"][:5]))
    b = solved("tolerance_1e-3.json", lang)
    check(b["solved"] is False, "%s: off by 1e-3 must fail" % lang)
    check(len(b["failed_cases"]) >= len(t["hidden"]) - 5, "%s: nearly every case must fail (%d of %d)" % (lang, len(b["failed_cases"]), len(t["hidden"])))
# boundary pair around atol 1e-6 (+ 1e-7 * |expected|, at most 1e-7 here since the expected values are below 1)
for lang in ("aval", "rust"):
    for sign in ("p", "m"):
        a = solved("tolerance_%s9e-7.json" % sign, lang)
        check(a["solved"] is True, "%s: an error of %s9e-7 must pass: %r" % (lang, "+" if sign == "p" else "-", a["failed_cases"][:5]))
        b = solved("tolerance_%s1.2e-6.json" % sign, lang)
        check(b["solved"] is False, "%s: an error of %s1.2e-6 must fail" % (lang, "+" if sign == "p" else "-"))
        check(len(b["failed_cases"]) >= len(t["hidden"]) // 2, "%s: most cases must fail at 1.2e-6 (%d of %d)" % (lang, len(b["failed_cases"]), len(t["hidden"])))
# atol == 0: exact comparison, so even 1e-8 fails while the exact reference still passes
d = tempfile.mkdtemp()
os.makedirs(os.path.join(d, "tasks"))
t0 = dict(t); t0["atol"] = 0
json.dump(t0, open(os.path.join(d, "tasks", "HE-002.json"), "w"))
for lang in ("aval", "rust"):
    a = solved("tolerance_1e-8.json", lang, os.path.join(d, "tasks"))
    check(a["solved"] is False, "%s: atol 0 must be an exact comparison" % lang)
    with tmp() as out:
        run(["--provider", "replay", "--responses", "reference", "--task", "HE-002", "--langs", lang, "--tasks", os.path.join(d, "tasks")], out)
        check(results(out)["tasks"]["HE-002"][lang]["solved"] is True, "%s: reference must pass exactly" % lang)
print("ok")
