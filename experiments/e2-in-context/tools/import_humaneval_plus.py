#!/usr/bin/env python3
"""Regenerate the hidden cases of the E2 task files from HumanEval+ (EvalPlus).

    python3 import_humaneval_plus.py TEST_JSONL [--tasks DIR] [--out DIR]

TEST_JSONL is EvalPlus's `test.jsonl`, pinned to dataset revision d32357cf319e50e9c8d8dab5ea876c72b0fd321b
(https://huggingface.co/datasets/evalplus/humanevalplus/resolve/d32357cf319e50e9c8d8dab5ea876c72b0fd321b/test.jsonl,
SHA-256 908377f1daf28dcb36846db73a5662b2e05a9907407c2696c89ad9d3b0b04492, Apache-2.0); the script refuses any
other file unless --no-verify is given. The file is never committed. Every `HE-NNN.json` found in --tasks (default: ../tasks next to this script) is read for
its metadata (id, statement, signatures, reference paths), its hidden cases are replaced by the HumanEval+
cases that map to Aval's types, and the result is written to --out (default: --tasks, in place).

The script is deterministic and idempotent: running it again on its own output reproduces the files byte
for byte. Python standard library only; no code from the dataset is executed (`inputs`/`results` are
parsed with ast.literal_eval).

Each case is kept or counted in `excluded` under exactly one reason (first match wins):

  unsupported_value    a value is a tuple, None, dict, set... or does not match the declared Aval type
  int_out_of_range     an integer outside the 64-bit signed range
  string_chars         a string with non-ASCII characters or control characters Aval cannot escape
  deliberate_departure the case depends on a documented departure of the reference/statement from upstream
  outside_statement    the inputs violate a precondition stated in the task statement
  budget               the case needs more than the interpreter allows a *natural* Aval solution: each case
                       gets its own 10M-step budget (batches get k times that) and recursion may nest at most
                       10,000 calls. For HE-015, 025, 031, 036, 049 and 056 the rule is that of the natural
                       recursive solution (see NATURAL below); for the other tasks it is the generic estimate
                       cost() against BUDGET, calibrated on the cheapest idiom, so those keep fewer cases than
                       a natural solution could handle.

Expected values are HumanEval+'s `results`, except for the tasks whose test calls `ref_func`
(HumanEval/14, /15): there the expected value is the reference function of the test, which is
re-implemented below (it is a one-liner each).

`atol` is the tolerance of the task: the `atol` of the test's assertion call; for tasks returning Float or
List<Float> with atol 0 it is 1e-6 (EvalPlus's rule). The check is abs(got - exp) <= atol when atol > 0,
else exact equality.
"""
import argparse
import ast
import hashlib
import json
import math
import os
import re
import sys

DATASET_REVISION = "d32357cf319e50e9c8d8dab5ea876c72b0fd321b"
DATASET_SHA256 = "908377f1daf28dcb36846db73a5662b2e05a9907407c2696c89ad9d3b0b04492"
BUDGET = 1000000
MAX_DEPTH = 9900      # the interpreter supports 10,000 nested calls; keep a margin for helper frames
ALLOWED_CONTROL = "\n\t"


def parse_test(test):
    inputs = re.search(r"^    inputs = (.*)$", test, re.M)
    results = re.search(r"^    results = (.*)$", test, re.M)
    call = re.search(r"assertion\(candidate\(\*inp\), (exp|ref_func\(\*inp\)), ([0-9.e+-]+)\)", test)
    if not inputs or not call:
        raise SystemExit("unrecognised test format")
    ins = ast.literal_eval(inputs.group(1))
    res = ast.literal_eval(results.group(1)) if results else None
    return ins, res, call.group(1) != "exp", float(call.group(2))


# ---- reference functions of the tests that use ref_func ---------------------------------------------
REF_FUNCS = {
    14: lambda s: [s[:i + 1] for i in range(len(s))],
    15: lambda n: " ".join(str(i) for i in range(n + 1)),
}


# ---- value conversion -----------------------------------------------------------------------------------
class Bad(Exception):
    def __init__(self, reason):
        self.reason = reason


def conv(v, ty):
    if ty == "Bool":
        if isinstance(v, bool):
            return v
        raise Bad("unsupported_value")
    if ty == "Int":
        if isinstance(v, bool) or not isinstance(v, int):
            raise Bad("unsupported_value")
        if not -2 ** 63 <= v < 2 ** 63:
            raise Bad("int_out_of_range")
        return v
    if ty == "Float":
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise Bad("unsupported_value")
        if isinstance(v, int) and abs(v) >= 2 ** 53:
            raise Bad("int_out_of_range")
        if not math.isfinite(v):
            raise Bad("unsupported_value")
        return float(v)
    if ty == "String":
        if not isinstance(v, str):
            raise Bad("unsupported_value")
        for c in v:
            if ord(c) > 126 or (ord(c) < 32 and c not in ALLOWED_CONTROL):
                raise Bad("string_chars")
        return v
    if ty.startswith("List<") and ty.endswith(">"):
        if not isinstance(v, list):
            raise Bad("unsupported_value")
        out, reason = [], None
        for x in v:
            try:
                out.append(conv(x, ty[5:-1]))
            except Bad as b:
                reason = reason or b.reason
        if reason:
            raise Bad(reason)
        return out
    raise SystemExit("unsupported declared type " + ty)


# ---- per-task rules ---------------------------------------------------------------------------------------
def n_chars(s):
    return len(s)


def trial_division_calls(n):
    """Calls of the natural recursive factorisation f(n, d): n < 2 -> []; d*d > n -> [n]; d | n -> f(n/d, d); else f(n, d+1)."""
    calls, d = 0, 2
    while calls <= MAX_DEPTH + 1:
        calls += 1
        if n < 2 or d * d > n:
            return calls
        if n % d == 0:
            n //= d
        else:
            d += 1
    return calls


def prime_calls(n):
    """Calls of the natural recursive primality test g(n, d): d*d > n -> true; d | n -> false; else g(n, d+1)."""
    calls, d = 0, 2
    while calls <= MAX_DEPTH + 1:
        calls += 1
        if d * d > n or n % d == 0:
            return calls
        d += 1
    return calls


# Natural solutions (one recursive call per element / per step; every step is a few dozen interpreter steps,
# so the 10,000-call recursion limit is reached long before the 10M-step budget, except for 015):
#   015 string_sequence(n) = string_sequence(n-1) + " " + n.to_string()       about 2.1 n^2 steps (measured:
#       the interpreter counts the bytes copied by each concatenation; n = 2000 needs 8.3M, n = 2179 exceeds 10M)
#   025 trial division, see trial_division_calls                              depth = calls
#   031 trial division up to sqrt(n), see prime_calls                         depth = calls
#   036 fizz_buzz(n) = digit sevens of n-1 + fizz_buzz(n-1)                   depth n + 1
#   049 modp(n, p) = modp(n-1, p) * 2 % p                                     depth n + 1
#   056 scan the characters carrying the depth                                depth len + 1
def natural_exceeds(num, a):
    if num == 15:
        return 2.2 * a[0] * a[0] + 20 * a[0] > 9500000
    if num in (36, 49):
        return a[0] + 1 > MAX_DEPTH
    if num == 25:
        return trial_division_calls(a[0]) > MAX_DEPTH
    if num == 31:
        return prime_calls(a[0]) > MAX_DEPTH
    if num == 56:
        return len(a[0]) + 1 > MAX_DEPTH
    return None


def cost(num, a):
    """Rough work estimate (interpreter steps) of the reference solution on the inputs `a`."""
    if num in (0, 3, 43):
        n = len(a[0]); return n * n * 8
    if num == 40:
        n = len(a[0]); return n * n * n * 8
    if num in (14, 18, 56):
        n = len(a[0]); return n * n * 6 + 100
    if num == 15:
        return (a[0] + 1) * 12
    if num == 24:
        return a[0] * 12
    if num == 25:
        return 40 * int(math.isqrt(a[0])) + 200
    if num == 31:
        return a[0] * 12
    if num == 36:
        return a[0] * 24
    if num == 46:
        return a[0] * 24
    if num == 49:
        return a[0] * 16
    if num == 76:
        return 200
    if num == 80:
        return len(a[0]) * 30
    if num in (26, 58):
        n = sum(len(x) for x in a); return n * n * 6
    if num in (62, 68, 70, 120):
        n = len(a[0]); return n * n * 6 + 200
    if num == 44:
        return 400
    return sum(len(x) if isinstance(x, (list, str)) else 1 for x in a) * 20 + 200


def departure(num, a, e):
    """Cases that depend on a documented departure from the original HumanEval (reason deliberate_departure).

    HE-044: the statement writes zero as the single digit "0"; the original HumanEval solution returned "".
    HE-064: the statement defines the word as non-empty; the original HumanEval solution raised on "".
    (HumanEval+ has since fixed both upstream; the cases stay excluded so that the task set does not depend
    on which version of the dataset is used.)
    """
    if num == 44:
        return a[0] == 0
    if num == 64:
        return a[0] == ""
    return False


def outside_statement(num, a):
    """Inputs violating a precondition stated by the task statement."""
    if num == 13:
        return not (a[0] > 0 and a[1] > 0)
    if num == 15:
        return a[0] < 0
    if num == 24:
        return a[0] < 2
    if num == 25:
        return a[0] < 2
    if num == 44:
        return a[0] < 0 or not 2 <= a[1] < 10
    if num == 49:
        return a[0] < 0 or a[1] < 2
    if num == 65:
        return a[0] < 0 or a[1] < 0
    if num == 76:
        return a[0] < 1 or a[1] < 1
    if num == 18:
        return len(a[1]) == 0
    if num == 36:
        return a[0] < 0
    if num == 46:
        return a[0] < 0
    if num == 47:
        return len(a[0]) == 0
    if num == 4:
        return len(a[0]) == 0
    if num == 2:
        return a[0] < 0
    if num == 120:
        return not 0 <= a[1] <= len(a[0])
    return False


def type_of_return_has_float(ty):
    return ty == "Float" or ty == "List<Float>"


# ---- main -----------------------------------------------------------------------------------------------
def js(v):
    return json.dumps(v)


def render(t):
    a = t["aval"]
    lines = ["{",
             '  "id": %s,' % js(t["id"]),
             '  "source": %s,' % js(t["source"]),
             '  "source_tests": %s,' % js(t["source_tests"]),
             '  "statement": %s,' % js(t["statement"]),
             '  "aval": {"name": %s, "params": %s, "returns": %s},' % (js(a["name"]), js(a["params"]), js(a["returns"])),
             '  "rust_signature": %s,' % js(t["rust_signature"]),
             '  "atol": %s,' % js(t["atol"]),
             '  "hidden": [']
    for i, h in enumerate(t["hidden"]):
        lines.append("    %s%s" % (js({"args": h["args"], "expected": h["expected"]}), "," if i + 1 < len(t["hidden"]) else ""))
    lines.append("  ],")
    lines.append('  "excluded": %s,' % js(t["excluded"]))
    lines.append('  "reference": %s' % js(t["reference"]))
    lines.append("}")
    return "\n".join(lines) + "\n"


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("test_jsonl")
    ap.add_argument("--tasks", default=os.path.join(here, "..", "tasks"))
    ap.add_argument("--out", default=None)
    ap.add_argument("--no-verify", action="store_true", help="do not check the SHA-256 of test_jsonl")
    args = ap.parse_args()
    if not args.no_verify:
        with open(args.test_jsonl, "rb") as f:
            digest = hashlib.sha256(f.read()).hexdigest()
        if digest != DATASET_SHA256:
            raise SystemExit("test.jsonl has SHA-256 %s, expected %s (evalplus/humanevalplus revision %s)"
                             % (digest, DATASET_SHA256, DATASET_REVISION))
    out_dir = args.out or args.tasks
    os.makedirs(out_dir, exist_ok=True)
    rows = {}
    with open(args.test_jsonl, encoding="utf-8") as f:
        for line in f:
            if line.strip():
                r = json.loads(line)
                rows[r["task_id"]] = r
    names = sorted(n for n in os.listdir(args.tasks) if re.fullmatch(r"HE-\d+\.json", n))
    for name in names:
        t = json.load(open(os.path.join(args.tasks, name), encoding="utf-8"))
        num = int(t["id"][3:])
        row = rows["HumanEval/%d" % num]
        ins, res, uses_ref, atol = parse_test(row["test"])
        if uses_ref:
            res = [REF_FUNCS[num](*i) for i in ins]
        assert len(ins) == len(res)
        ptypes = [p[1] for p in t["aval"]["params"]]
        rtype = t["aval"]["returns"]
        if atol == 0 and type_of_return_has_float(rtype):
            atol = 1e-6
        hidden, excluded = [], {}
        seen = set()
        for i, e in zip(ins, res):
            reason = None
            try:
                if len(i) != len(ptypes):
                    raise Bad("unsupported_value")
                args_c = [conv(v, ty) for v, ty in zip(i, ptypes)]
                exp_c = conv(e, rtype)
            except Bad as b:
                reason = b.reason
            if reason is None and departure(num, i, e):
                reason = "deliberate_departure"
            if reason is None and outside_statement(num, i):
                reason = "outside_statement"
            over = natural_exceeds(num, i)
            if reason is None and (over if over is not None else cost(num, i) > BUDGET):
                reason = "budget"
            if reason:
                excluded[reason] = excluded.get(reason, 0) + 1
                continue
            key = json.dumps([args_c, exp_c])
            if key in seen:      # EvalPlus inputs are already unique; keep the guard for reproducibility
                continue
            seen.add(key)
            hidden.append({"args": args_c, "expected": exp_c})
        t["source_tests"] = ("HumanEval+ (EvalPlus, Liu et al. 2023, Apache-2.0, https://github.com/evalplus/evalplus), "
                             "HumanEval/%d from https://huggingface.co/datasets/evalplus/humanevalplus" % num)
        t["atol"] = 0 if atol == 0 else atol
        t["hidden"] = hidden
        t["excluded"] = dict(sorted(excluded.items()))
        with open(os.path.join(out_dir, name), "w", encoding="utf-8", newline="\n") as f:
            f.write(render(t))
        print("%s kept=%d excluded=%s atol=%s" % (t["id"], len(hidden), t["excluded"], atol))


if __name__ == "__main__":
    main()
