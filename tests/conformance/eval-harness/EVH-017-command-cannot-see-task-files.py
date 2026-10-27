#!/usr/bin/env python3
# scenario: EVH-017
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

tdir = os.path.join(REPO, "experiments", "e2-in-context", "tasks")
task = json.load(open(os.path.join(tdir, "HE-003.json")))
def answers(extra_cmd, env=None, extra_args=()):
    with tmp() as out:
        run(["--provider", "command", "--command", extra_cmd, "--task", "HE-003", "--langs", "aval", "--repairs", "0"] + list(extra_args), out, env=env)
        return results(out)["tasks"]["HE-003"]["aval"]["attempts"][0].get("answer") or ""
for ans in (answers("ls; cat tasks/*.json"), answers("ls; cat tasks/*.json; true")):
  for needle in (task["statement"], "HumanEval", "hidden", "rust_signature", "HE-003.json", "below_zero"):
    check(needle not in ans, "the recorded answer contains task file content %r: %r" % (needle, ans[:200]))
# the environment is limited (secrets are not passed unless named by --pass-env)
env = {"E2_SECRET_FOR_TEST": "hunter2-xyz"}
check("hunter2-xyz" not in answers("env", env), "secret leaked into the command environment")
check("hunter2-xyz" in answers("env", env, ["--pass-env", "E2_SECRET_FOR_TEST"]), "--pass-env variable not passed")
print("ok")
