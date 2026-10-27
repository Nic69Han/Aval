#!/usr/bin/env python3
# scenario: EVH-013
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

with tmp() as out:
    run(["--provider", "replay", "--responses", fx("rust_exit_early.json"), "--task", "HE-003", "--langs", "rust", "--repairs", "0"], out)
    a = results(out)["tasks"]["HE-003"]["rust"]
    check(a["solved"] is False and a["pass_at_1"] is False, "a candidate that exits(0) must not be solved: %r" % a)
    check(a["attempts"][0]["outcome"] == "failed", "attempt must fail")
# twin: the unmodified Rust reference is solved by the same harness
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--task", "HE-003", "--langs", "rust"], out)
    check(results(out)["tasks"]["HE-003"]["rust"]["solved"] is True, "reference must be solved")
print("ok")
