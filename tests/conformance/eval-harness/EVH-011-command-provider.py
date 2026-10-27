#!/usr/bin/env python3
# scenario: EVH-011
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

with tmp() as out:
    run(["--provider", "command", "--command", "cat " + shlex.quote(fx("reference_answer.txt")), "--task", "HE-003", "--langs", "aval"], out)
    a = results(out)["tasks"]["HE-003"]["aval"]
    check(a["solved"] is True and a["solved_at"] == 1, "task must be solved at attempt 1")
    for att in a["attempts"]:
        check(att.get("tokens_source") and att["tokens_source"] != "provider", "token source %r" % att.get("tokens_source"))
    check(results(out)["provider"] == "command", "provider recorded as %r" % results(out)["provider"])
print("ok")
