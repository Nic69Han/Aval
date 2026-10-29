#!/usr/bin/env python3
"""Combine the results.json files of several runs of one E2 configuration.

    python3 aggregate_runs.py --out DIR RESULTS1 RESULTS2 ...

Reads only the `results.json` files (the harness is not involved) and writes DIR/AGGREGATE.md and
DIR/aggregate.json: per-run pass@1 per language, the mean, the sample standard deviation (n-1; null below 3 runs),
the flip rate (tasks whose first-attempt outcome differs between runs, divided by the task count; tasks with a
provider error in any run are excluded from numerator and denominator) and the pre-registered criterion
(Aval pass@1 >= 0.8 x Rust pass@1) evaluated on the mean and, separately, on every run.

The runs must agree on task ids, skill SHA-256, repairs, provider and model, and none may carry the verdict
INCONCLUSIVE; otherwise the exit status is non-zero, the message names the file or field and nothing is written.
Per-run pass@1 is read from `metrics.<lang>.pass_at_1` and is not recomputed.
"""
import argparse
import hashlib
import json
import math
import os
import shutil
import statistics
import sys
import tempfile
from fractions import Fraction

LANGS = ("aval", "rust")
SAME = ("skill_sha256", "repairs", "provider", "model")
FACTOR = Fraction(4, 5)
MIN_RUNS = 3


def reject(msg):
    print("aggregate_runs: " + msg, file=sys.stderr)
    sys.exit(1)


def load(path):
    """Return (parsed JSON, SHA-256 of the file bytes)."""
    try:
        with open(path, "rb") as f:
            raw = f.read()
        return json.loads(raw.decode("utf-8")), hashlib.sha256(raw).hexdigest()
    except (OSError, ValueError) as e:
        reject("cannot read %s: %s" % (path, " ".join(str(e).split())))


def is_num(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)


def exact(x):
    """Exact rational value of a stored rate (k/n written as a float): shortest repr, then the nearest simple fraction."""
    return Fraction(repr(x)).limit_denominator(10 ** 6)


def check_shape(p, r):
    if not isinstance(r, dict) or "tasks" not in r or "metrics" not in r:
        reject("%s is not a results.json file" % p)
    if not isinstance(r["tasks"], dict) or not r["tasks"]:
        reject("%s: 'tasks' must be a non-empty object" % p)
    if not isinstance(r["metrics"], dict):
        reject("%s: 'metrics' must be an object" % p)
    for lang in LANGS:
        m = r["metrics"].get(lang)
        if not isinstance(m, dict):
            reject("%s: missing metrics.%s" % (p, lang))
        v = m.get("pass_at_1")
        if not is_num(v) or not 0 <= v <= 1:
            reject("%s: metrics.%s.pass_at_1 must be a number between 0 and 1" % (p, lang))
    for t, task in r["tasks"].items():
        for lang in LANGS:
            arm = task.get(lang) if isinstance(task, dict) else None
            if not isinstance(arm, dict):
                reject("%s: task %s has no %s arm" % (p, t, lang))
            if not isinstance(arm.get("attempts", []), list) or not all(isinstance(a, dict) for a in arm.get("attempts", [])):
                reject("%s: task %s.%s has malformed attempts" % (p, t, lang))
            if not isinstance(arm.get("pass_at_1"), bool):
                reject("%s: task %s.%s.pass_at_1 must be a boolean" % (p, t, lang))


def check_runs(paths, runs, digests):
    seen_path, seen_hash = {}, {}
    for p, d in zip(paths, digests):
        rp = os.path.realpath(p)
        if rp in seen_path:
            reject("duplicate input: %s is the same file as %s" % (p, seen_path[rp]))
        seen_path[rp] = p
        if d in seen_hash:
            reject("duplicate input: %s has identical content to %s" % (p, seen_hash[d]))
        seen_hash[d] = p
    for p, r in zip(paths, runs):
        check_shape(p, r)
        v = r.get("verdict")
        if isinstance(v, dict) and v.get("result") == "INCONCLUSIVE":
            reject("%s has the verdict INCONCLUSIVE and carries no usable pass rate" % p)
    first = runs[0]
    for p, r in zip(paths[1:], runs[1:]):
        if sorted(r["tasks"]) != sorted(first["tasks"]):
            reject("task ids differ between %s and %s" % (paths[0], p))
        for field in SAME:
            if r.get(field) != first.get(field):
                reject("%s differs between %s (%r) and %s (%r)" % (field, paths[0], first.get(field), p, r.get(field)))


def has_error(arm):
    return arm.get("outcome") == "provider_error" or any(a.get("outcome") == "provider_error" for a in arm.get("attempts", []))


def flip_rate(runs, lang):
    ids = sorted(runs[0]["tasks"])
    flips = counted = 0
    for t in ids:
        arms = [r["tasks"][t][lang] for r in runs]
        if any(has_error(a) for a in arms):
            continue
        counted += 1
        if len({bool(a.get("pass_at_1")) for a in arms}) > 1:
            flips += 1
    return (flips / counted if counted else None), flips, counted


def fmt(x):
    return "n/a" if x is None else "%.4f" % x


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--out", required=True, help="directory for AGGREGATE.md and aggregate.json")
    ap.add_argument("results", nargs="+", help="results.json files of runs of one configuration")
    args = ap.parse_args()

    loaded = [load(p) for p in args.results]
    runs = [x[0] for x in loaded]
    check_runs(args.results, runs, [x[1] for x in loaded])
    n = len(runs)

    langs = {}
    flip_info = {}
    exact_vals = {}
    for lang in LANGS:
        vals = [r["metrics"][lang]["pass_at_1"] for r in runs]
        rate, flips, counted = flip_rate(runs, lang)
        flip_info[lang] = (flips, counted)
        exact_mean = sum(exact(v) for v in vals) / n
        exact_vals[lang] = ([exact(v) for v in vals], exact_mean)
        langs[lang] = {"pass_at_1": vals, "mean": float(exact_mean),
                       "stdev": statistics.stdev(vals) if n >= MIN_RUNS else None, "flip_rate": rate}
    aval, rust = langs["aval"]["pass_at_1"], langs["rust"]["pass_at_1"]
    (ea, ma), (er, mr) = exact_vals["aval"], exact_vals["rust"]
    mean_ok = ma >= FACTOR * mr
    every_ok = all(a >= FACTOR * b for a, b in zip(ea, er))
    verdict = {"mean": "PASS" if mean_ok else "FAIL", "every_run": every_ok}
    agg = {"runs": n, "languages": langs, "verdict": verdict}

    first = runs[0]
    lines = ["# Aggregate of %d runs" % n, "",
             "- Task set: %s, %d tasks" % (first.get("task_set"), len(first["tasks"])),
             "- Skill SHA-256: %s" % first.get("skill_sha256"),
             "- Provider: %s, model: %s, repairs: %s" % (first.get("provider"), first.get("model"), first.get("repairs")),
             "", "| Run | File | Aval pass@1 | Rust pass@1 |", "|---|---|---|---|"]
    for i, p in enumerate(args.results):
        lines.append("| %d | %s | %s | %s |" % (i + 1, p, fmt(aval[i]), fmt(rust[i])))
    lines += ["", "| Language | Mean pass@1 | Std. dev. (sample) | Flip rate |", "|---|---|---|---|"]
    for lang in LANGS:
        L = langs[lang]
        lines.append("| %s | %s | %s | %s |" % (lang, fmt(L["mean"]), fmt(L["stdev"]), fmt(L["flip_rate"])))
    lines.append("")
    for lang in LANGS:
        flips, counted = flip_info[lang]
        lines.append("Flip rate %s: %d of %d tasks differ in first-attempt outcome (tasks with a provider error excluded)%s."
                     % (lang, flips, counted, "" if counted else "; no countable task, flip rate n/a"))
    if n < MIN_RUNS:
        lines += ["", "WARNING: fewer than %d runs; the standard deviation is not meaningful." % MIN_RUNS]
        print("warning: fewer than %d runs; the standard deviation is not meaningful" % MIN_RUNS, file=sys.stderr)
    if first.get("model") is None:
        lines += ["", "NOTE: Model identity not verified (every run records model null)."]
        print("note: Model identity not verified (every run records model null)", file=sys.stderr)
    lines += ["", "Criterion (pre-registered): Aval pass@1 >= 0.8 x Rust pass@1, applied to the mean pass@1."]
    if mr == 0:
        lines.append("Rust pass@1 is 0: criterion holds trivially")
    lines += ["Verdict on mean: " + verdict["mean"],
              "Every run satisfies the criterion: " + ("yes" if every_ok else "no"), ""]

    write_outputs(args.out, "\n".join(lines), json.dumps(agg, indent=2, sort_keys=True) + "\n")


def write_outputs(out, report, data):
    """All-or-nothing: both temp files are written before either target is replaced; any OSError exits 1."""
    targets = [(os.path.join(out, "AGGREGATE.md"), report), (os.path.join(out, "aggregate.json"), data)]
    temps, backups, replaced, created = [], {}, [], None
    try:
        if not os.path.isdir(out):
            created = out
            while not os.path.exists(os.path.dirname(created) or "."):
                created = os.path.dirname(created)
            os.makedirs(out, exist_ok=True)
        for target, text in targets:
            if os.path.isdir(target):
                raise IsADirectoryError("%s is a directory" % target)
            if os.path.exists(target):
                with open(target, "rb") as f:
                    backups[target] = f.read()
        for target, text in targets:
            fd, tmp = tempfile.mkstemp(prefix=".aggregate-", suffix=".tmp", dir=out)
            temps.append(tmp)
            with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as f:
                f.write(text)
        for (target, _), tmp in zip(targets, temps):
            os.replace(tmp, target)
            replaced.append(target)
    except OSError as e:
        for tmp in temps:
            try:
                os.unlink(tmp)
            except OSError:
                pass
        for target in replaced:
            try:
                if target in backups:
                    with open(target, "wb") as f:
                        f.write(backups[target])
                else:
                    os.unlink(target)
            except OSError:
                pass
        if created is not None and not replaced:
            try:
                shutil.rmtree(created) if created != out else os.rmdir(out)
            except OSError:
                pass
        reject("cannot write output in %s: %s" % (out, " ".join(str(e).split())))
    print("wrote %s and %s" % (targets[0][0], targets[1][0]))


if __name__ == "__main__":
    main()
