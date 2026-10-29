#!/usr/bin/env python3
# scenario: EVH-048
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _aggregate import *

base = solved_map(0, fail_ids=["MB-010"])


def runs(w):
    return [write_run(w, "r%d" % i, make_results(base, base)) for i in (1, 2, 3)]


def snapshot(out):
    """Relative path -> (kind, bytes) for everything under out (or the file itself)."""
    if os.path.isfile(out):
        return {".": ("file", open(out, "rb").read())}
    snap = {}
    for root, dirs, files in os.walk(out):
        for d in dirs:
            snap[os.path.relpath(os.path.join(root, d), out)] = ("dir", None)
        for f in files:
            snap[os.path.relpath(os.path.join(root, f), out)] = ("file", open(os.path.join(root, f), "rb").read())
    return snap


def fail_clean(label, prepare):
    with workdir() as w:
        paths = runs(w)
        out = os.path.join(w, "out")
        prepare(out)
        before = snapshot(out)
        r = aggregate(out, paths)
        err = r.stderr
        check(r.returncode == 1, "%s: exit status must be 1, got %d (stdout %r, stderr %r)" % (label, r.returncode, r.stdout[-200:], err[-300:]))
        check("Traceback" not in err and "Traceback" not in r.stdout, "%s: no traceback allowed: %r" % (label, err[-500:]))
        check(len(err.strip().splitlines()) == 1, "%s: the message must be one line, got %r" % (label, err))
        check(snapshot(out) == before, "%s: existing output must be left unchanged and nothing new created (before %r, after %r)" % (label, sorted(before), sorted(snapshot(out))))


def out_is_file(out):
    open(out, "w").write("not a directory\n")


def report_is_dir(out):
    os.makedirs(os.path.join(out, "AGGREGATE.md"))


def json_is_dir(out):
    os.makedirs(os.path.join(out, "aggregate.json"))
    open(os.path.join(out, "AGGREGATE.md"), "wb").write(b"previous report, must stay byte-identical\n")


fail_clean("--out is an existing file", out_is_file)
fail_clean("AGGREGATE.md is a directory", report_is_dir)
fail_clean("aggregate.json is a directory, AGGREGATE.md exists", json_is_dir)

# twin: a normal fresh --out is accepted and gets both files
with workdir() as w:
    out = os.path.join(w, "out")
    r = aggregate(out, runs(w))
    check(r.returncode == 0, "a fresh --out must be accepted: " + (r.stdout + r.stderr)[-300:])
    load_out(out)
print("ok")
