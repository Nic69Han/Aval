#!/usr/bin/env python3
# scenario: EVH-008
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

def one(resp, extra=("--repairs", "0")):
    with tmp() as out:
        run(["--provider", "replay", "--responses", fx(resp), "--task", "HE-003", "--langs", "aval"] + list(extra), out)
        return results(out)["tasks"]["HE-003"]["aval"]
a = one("no_code.json", ("--repairs", "1"))
check(a["attempts"][0]["reason"] == "no_code", "attempt 1 reason: %r" % a["attempts"][0]["reason"])
check(a["attempts"][0]["outcome"] == "failed", "attempt 1 must fail")
check(a["solved"] is True and a["solved_at"] == 2, "the second (good) answer must solve the task")
# an answer whose only fenced block is of another language is also no_code
a = one("wrong_language_block.json")
check(a["attempts"][0]["reason"] == "no_code" and a["solved"] is False, "wrong-language block: %r" % a["attempts"][0])
# the first block of the expected language is taken (the second, correct one is ignored)
a = one("first_block_wins.json")
check(a["solved"] is False and a["attempts"][0]["reason"] == "hidden_failed", "first block must be used: %r" % a["attempts"][0])
# a block of another language before the right one is skipped
a = one("other_language_first.json")
check(a["solved"] is True and a["solved_at"] == 1, "python block must be skipped")
print("ok")
