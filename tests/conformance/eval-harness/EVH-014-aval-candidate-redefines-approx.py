#!/usr/bin/env python3
# scenario: EVH-014
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

with tmp() as out:
    run(["--provider", "replay", "--responses", fx("approx_redefined.json"), "--task", "HE-002", "--langs", "aval", "--repairs", "0"], out)
    a = results(out)["tasks"]["HE-002"]["aval"]
    check(a["solved"] is False and a["pass_at_1"] is False, "redefining approx must not solve the task: %r" % a)
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--task", "HE-002", "--langs", "aval"], out)
    check(results(out)["tasks"]["HE-002"]["aval"]["solved"] is True, "reference must be solved")
print("ok")
