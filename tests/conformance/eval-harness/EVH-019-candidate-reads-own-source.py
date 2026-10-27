#!/usr/bin/env python3
# scenario: EVH-019
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

with tmp() as out:
    run(["--provider", "replay", "--responses", fx("rust_forges_attestation.json"), "--task", "HE-003", "--langs", "rust", "--repairs", "0"], out)
    a = results(out)["tasks"]["HE-003"]["rust"]
    check(a["solved"] is False and a["pass_at_1"] is False, "a candidate printing attestation lines read from main.rs must not be solved: %r" % a)
print("ok")
