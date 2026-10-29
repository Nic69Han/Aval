#!/usr/bin/env python3
# scenario: EVH-047
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _aggregate import *

base = solved_map(0, fail_ids=["MB-010"])
MSG = "Model identity not verified"


def go(model):
    with workdir() as w:
        paths = [write_run(w, "r%d" % i, make_results(base, base, model=model)) for i in (1, 2, 3)]
        out = os.path.join(w, "out")
        r = aggregate(out, paths)
        check(r.returncode == 0, "runs rejected: " + (r.stdout + r.stderr)[-500:])
        md, _ = load_out(out)
        return md, r.stderr


md, err = go(None)
check(MSG in md, "AGGREGATE.md must state %r when every run records model null" % MSG)
check(MSG in err, "stderr must state %r when every run records model null, got %r" % (MSG, err))
# negative twin: a recorded model needs no such statement
md, err = go("m-1")
check(MSG not in md and MSG not in err, "a recorded model must not produce the note")
print("ok")
