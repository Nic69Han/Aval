#!/usr/bin/env python3
# scenario: EVH-032
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *
MBPP = os.path.join(REPO, "experiments", "e2-in-context", "tasks-mbpp")

import ast
try:
    import pyarrow.parquet as pq
except ImportError:
    fail("PYARROW: this test needs pyarrow in the Python that runs it (pip install pyarrow==26.0.0)")
rows = {r["task_id"]: r for r in pq.read_table(fetch_mbpp_parquet()).to_pylist()}

def depth(v):
    if isinstance(v, (list, tuple)):
        return 1 + max((depth(x) for x in v), default=0)
    return 0

files = sorted(f for f in os.listdir(MBPP) if re.fullmatch(r"MB-\d+\.json", f))
check(len(files) >= 150, "only %d task files" % len(files))
checked = 0
for f in files:
    t = json.load(open(os.path.join(MBPP, f)))
    check(not re.search(r"python", t["statement"], re.I), "%s: the statement names a programming language: %r" % (f, t["statement"][:120]))
    num = int(t["id"][3:])
    entry = re.search(r"entry point (\w+)", t["source"]).group(1)
    first = rows[num]["test_list"][0].strip()
    try:
        tree = ast.parse(first)
        call = next(n for n in ast.walk(tree) if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == entry)
        orig = [depth(ast.literal_eval(a)) for a in call.args]
    except (SyntaxError, StopIteration, ValueError):
        continue            # first assertion not a plain literal call: not checkable here
    params = t["aval"]["params"]
    check(len(orig) == len(params), "%s: %d original arguments, %d parameters" % (f, len(orig), len(params)))
    for (name, ty), d in zip(params, orig):
        check(ty.count("List<") == d, "%s: parameter %s has type %s (nesting %d) but the original MBPP assertion has depth %d: %s" %
              (f, name, ty, ty.count("List<"), d, first[:100]))
    checked += 1
check(checked >= 0.9 * len(files), "only %d of %d tasks could be compared with their original assertion" % (checked, len(files)))
print("ok (%d tasks, %d compared)" % (len(files), checked))
