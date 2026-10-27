#!/usr/bin/env python3
# scenario: EVH-018
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

with tmp() as out:
    run(["--provider", "replay", "--responses", "reference", "--task", "HE-002", "--task", "HE-003", "--task", "HE-013"], out)
    md = open(os.path.join(out, "RESULTS.md"), encoding="utf-8").read()
    for label in ("tokens/solved (with system)", "tokens/solved (without system)"):
        check(label in md, "RESULTS.md lacks %r" % label)
    res = results(out)
    for lang in ("aval", "rust"):
        m = res["metrics"][lang]
        check(m["tokens_per_solved"] and m["tokens_per_solved_without_system"], "%s token figures missing" % lang)
        check(m["tokens_per_solved_without_system"] < m["tokens_per_solved"], "%s: without-system figure must be smaller" % lang)
    # the Aval system prompt (the whole reference) is much larger than the Rust one
    a, r = res["metrics"]["aval"], res["metrics"]["rust"]
    check(a["tokens_per_solved"] - a["tokens_per_solved_without_system"] > r["tokens_per_solved"] - r["tokens_per_solved_without_system"], "system share must differ between the arms")
print("ok")
