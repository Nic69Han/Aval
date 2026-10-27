#!/usr/bin/env python3
# scenario: EVH-029
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *
MBPP = os.path.join(REPO, "experiments", "e2-in-context", "tasks-mbpp")

files = sorted(f for f in os.listdir(MBPP) if re.fullmatch(r"MB-\d+\.json", f))
check(len(files) >= 150, "only %d task files" % len(files))
for f in files:
    t = json.load(open(os.path.join(MBPP, f)))
    lines = [l for l in t["statement"].splitlines() if l.startswith("Example:")]
    check(len(lines) == 1, "%s: expected one Example: line, found %d" % (f, len(lines)))
    check(t["statement"].count("Example:") == 1, f + ": Example: appears more than once")
    m = re.fullmatch(r"Example: (\w+)\((.*)\) returns (.*)", lines[0])
    check(m, "%s: unparsable Example line %r" % (f, lines[0]))
    check(m.group(1) == t["aval"]["name"], f + ": the example must call the task function")
    args = json.loads("[" + m.group(2) + "]")
    value = json.loads(m.group(3))
    check(len(args) == len(t["aval"]["params"]), f + ": example arity")
    key = lambda a, v: json.dumps([a, v], sort_keys=True)
    shown = key(args, value)
    for h in t["hidden"]:
        check(key(h["args"], h["expected"]) != shown, "%s: the shown example is also a hidden case" % f)
        check(h["args"] != args, "%s: the shown arguments %r appear among the hidden cases" % (f, args))
    check(t["excluded"].get("shown_in_statement") == 1, "%s: excluded.shown_in_statement = %r" % (f, t["excluded"].get("shown_in_statement")))
    check(len(t["hidden"]) >= 20, f + ": fewer than 20 hidden cases")
print("ok (%d tasks)" % len(files))
