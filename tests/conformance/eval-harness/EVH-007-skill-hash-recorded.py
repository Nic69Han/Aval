#!/usr/bin/env python3
# scenario: EVH-007
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

skill = skill_text()
want = hashlib.sha256(skill).hexdigest()
with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--task", "HE-003"], out)
    got = results(out).get("skill_sha256")
    check(got == want, "skill_sha256 %r != %r" % (got, want))
# the system prompt is exactly the skill text
with tmp() as out, tmp() as rec:
    run(["--provider", "command", "--command", recorder_cmd(rec, "recorder_answers_e301"), "--task", "HE-003", "--langs", "aval"], out)
    check(skill.decode("utf-8") in prompts(rec)[0], "the prompt must contain the exact output of `aval skill print`")
print("ok")
