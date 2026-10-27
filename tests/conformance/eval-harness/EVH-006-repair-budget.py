#!/usr/bin/env python3
# scenario: EVH-006
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

def attempts(extra):
    with tmp() as out:
        run(["--provider", "replay", "--responses", fx("never_right.json"), "--task", "HE-003", "--langs", "aval"] + extra, out)
        a = results(out)["tasks"]["HE-003"]["aval"]
        check(a["solved"] is False, "task must not be solved")
        return len(a["attempts"])
check(attempts(["--repairs", "2"]) == 3, "--repairs 2 must record exactly 3 attempts")
check(attempts(["--repairs", "0"]) == 1, "--repairs 0 must record exactly 1 attempt")
check(attempts([]) == 4, "default repairs (3) must record exactly 4 attempts")
print("ok")
