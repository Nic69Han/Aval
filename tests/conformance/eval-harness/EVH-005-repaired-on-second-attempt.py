#!/usr/bin/env python3
# scenario: EVH-005
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

with tmp() as out:
    run(["--provider", "replay", "--responses", fx("repair_e301.json"), "--task", "HE-003", "--langs", "aval"], out)
    a = results(out)["tasks"]["HE-003"]["aval"]
    check(a["solved"] is True and a["solved_at"] == 2, "solved_at = %r" % a["solved_at"])
    check(a["compile_at_1"] is False, "compile_at_1 must be false")
    check(a["pass_at_1"] is False, "pass_at_1 must be false")
    check(len(a["attempts"]) == 2, "two attempts expected")
    check(a["attempts"][0]["outcome"] == "failed" and a["attempts"][0]["reason"] == "compile_error", "attempt 1: %r" % a["attempts"][0])
    check(a["attempts"][0]["diagnostics"] >= 1, "attempt 1 must count the E301 diagnostic")
    check(a["attempts"][1]["outcome"] == "solved", "attempt 2 must be solved")
# the diagnostics go back verbatim
with tmp() as out, tmp() as rec:
    run(["--provider", "command", "--command", recorder_cmd(rec, "recorder_answers_e301"), "--task", "HE-003", "--langs", "aval"], out)
    ps = prompts(rec)
    check(len(ps) == 2, "expected 2 prompts")
    line = re.compile(r'\{"code":"E301","at":\[\d+,\d+\]')   # the reference only shows "at":[line,col]
    check(not line.search(ps[0]), "first prompt must not contain a diagnostic")
    check(line.search(ps[1]), "repair prompt must contain the E301 JSON line verbatim")
print("ok")
