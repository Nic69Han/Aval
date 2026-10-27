#!/usr/bin/env python3
# scenario: EVH-010
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

with tmp() as out:
    run(["--provider", "replay", "--responses", fx("rust_repair.json"), "--task", "HE-003", "--langs", "rust"], out)
    a = results(out)["tasks"]["HE-003"]["rust"]
    check(a["solved"] is True and a["solved_at"] == 2, "solved_at = %r" % a["solved_at"])
    check(a["compile_at_1"] is False and a["pass_at_1"] is False, "first answer does not compile")
    check(a["attempts"][0]["reason"] == "compile_error" and a["attempts"][0]["compile_ok"] is False, "attempt 1: %r" % a["attempts"][0])
with tmp() as out, tmp() as rec:
    run(["--provider", "command", "--command", recorder_cmd(rec, "recorder_answers_rust"), "--task", "HE-003", "--langs", "rust"], out)
    ps = prompts(rec)
    check(len(ps) == 2, "expected 2 prompts")
    check("E0308" not in ps[0] and "E0308" in ps[1], "the rustc error must be sent back in the repair prompt")
print("ok")
