#!/usr/bin/env python3
# scenario: EVH-046
import os, shutil, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _aggregate import *

base = solved_map(0, fail_ids=["MB-010"])


def good():
    return make_results(base, base)


def reject(label, build, field=None):
    """build(w) -> (paths, names) where names are path strings of which at least one must appear in the message."""
    with workdir() as w:
        paths, names = build(w)
        out = os.path.join(w, "out")
        r = aggregate(out, paths)
        err = r.stderr
        check(r.returncode == 1, "%s: exit status must be 1, got %d (stdout %r, stderr %r)" % (label, r.returncode, r.stdout[-200:], err[-300:]))
        check("Traceback" not in err and "Traceback" not in r.stdout, "%s: no traceback allowed: %r" % (label, err[-500:]))
        check(len(err.strip().splitlines()) == 1, "%s: the message must be one line, got %r" % (label, err))
        check(any(n in err for n in names), "%s: the message must name the file (%s), got %r" % (label, names, err))
        if field:
            check(field in err, "%s: the message must mention the field %r (rejected for the wrong reason?), got %r" % (label, field, err))
        check(out_untouched(out), "%s: nothing may be written to --out" % label)


def duplicate_path(w):
    p = write_run(w, "r1", good())
    return [p, p, p], [p]


def duplicate_content(w):
    res = good()
    ps = [write_run(w, "r%d" % i, res, distinct=False) for i in (1, 2, 3)]
    return ps, ps


def with_bad(mutate=None, raw=None):
    def build(w):
        p1 = write_run(w, "r1", good())
        res = good()
        if mutate:
            res = mutate(res)
        p2 = write_run(w, "r2", res)
        if raw is not None:
            open(p2, "w").write(raw)
        p3 = write_run(w, "r3", good())
        return [p1, p2, p3], [p2]
    return build


def drop(o, *keys):
    for k in keys[:-1]:
        o = o[k]
    del o[keys[-1]]
    return o


def set_(o, v, *keys):
    for k in keys[:-1]:
        o = o[k]
    o[keys[-1]] = v
    return o


def no_rust_metrics(r):
    drop(r, "metrics", "rust")
    return r


def no_rust_arm(r):
    drop(r, "tasks", "MB-001", "rust")
    return r


def str_pass(r):
    set_(r, "0.9", "metrics", "aval", "pass_at_1")
    return r


def list_tasks(r):
    r["tasks"] = []
    return r


DELETE = object()


def field_case(where, value):
    """Mutate only one pass_at_1 field of a good results dict and return the whole dict."""
    def m(r):
        o = r
        for k in where[:-1]:
            o = o[k]
        if value is DELETE:
            del o[where[-1]]
        else:
            o[where[-1]] = value
        return r
    return m


reject("duplicate path", duplicate_path)
reject("identical content", duplicate_content)
reject("invalid JSON", with_bad(raw="{not json"))
reject("missing metrics.rust", with_bad(no_rust_metrics))
reject("task without the rust arm", with_bad(no_rust_arm))
reject("pass_at_1 as a string", with_bad(str_pass))
reject("tasks as a list", with_bad(list_tasks))
LOCATIONS = [("metrics", "aval", "pass_at_1"), ("metrics", "rust", "pass_at_1"),
             ("tasks", "MB-001", "aval", "pass_at_1"), ("tasks", "MB-001", "rust", "pass_at_1")]
VALUES = [("deleted", DELETE), ("string 'false'", "false"), ("string 'true'", "true"), ("NaN", float("nan")), ("null", None)]
for where in LOCATIONS:
    for name, value in VALUES + ([("number 1", 1)] if where[0] == "tasks" else []):   # 1 is a valid rate in metrics
        # sanity: the mutation changes exactly one field of an otherwise good results dict
        probe = field_case(where, value)(good())
        check(isinstance(probe, dict) and probe.get("schema") == 1, "test setup: mutation must return the results dict")
        reject("%s pass_at_1 %s" % (".".join(where[:-1]), name), with_bad(field_case(where, value)), field="pass_at_1")
# twin: three distinct valid files are accepted
with workdir() as w:
    r = aggregate(os.path.join(w, "out"), [write_run(w, "r%d" % i, good()) for i in (1, 2, 3)])
    check(r.returncode == 0, "three distinct valid files must be accepted: " + r.stderr[-300:])
print("ok")
