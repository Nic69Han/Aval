#!/usr/bin/env python3
"""Import the MBPP+ task set (EvalPlus) into E2 task files.

    python3 import_mbpp_plus.py MBPPPLUS_PARQUET [--out DIR] [--excluded FILE] [--report]

MBPPPLUS_PARQUET is EvalPlus's `mbppplus` dataset (evalplus/mbppplus, revision
b2d74c91837c3f2a20c1299ae98133cbe7cfa077, file data/test-*.parquet, Apache-2.0; MBPP itself: Austin et al. 2021,
CC-BY-4.0). The script refuses any file whose SHA-256 is not

    dc20030b3788fccf617444edcb34138ef13d7e4fafd17bfcb8c1279dbb12399b

(`--no-verify` disables the check). It needs `pyarrow` (development only) and runs no code from the dataset:
`inputs`/`results` are parsed from the `test` field with the `ast` module and evaluated by a tiny literal
evaluator.

For every MBPP+ task whose test is a plain `assertion(f(*inp), exp, atol)` comparison the script infers the Aval
types, keeps the cases that map to them and writes `MB-NNN.json` (NNN = MBPP task id) into --out (default:
`../tasks-mbpp` next to this script) when at least MIN_CASES cases remain. Tasks listed in EXCLUDED.md
(`| MB-NNN | reason |` rows; default `--excluded`: `<out>/EXCLUDED.md`) are not written: they are the tasks whose
reference could not be written within Aval v0.1.

Type inference. A parameter's list nesting depth is that of the corresponding argument in the first assertion of
the original MBPP `test_list` (EVH-032); ties between shapes are broken by that shape. A parameter (or the result)
has the type of the majority of its cases among Int, Float, Bool,
String and List<T> (T inferred the same way over all elements, lists may nest); an Int is accepted where Float
is inferred. A case is excluded when any of its values has another type, is a tuple/dict/set/None, or mixes
element types. Function and parameter names: the entry point and the canonical solution's parameters, converted to
snake_case and renamed when they are Aval or Rust keywords.

Reasons counted in `excluded` (first match wins): type_mismatch, unsupported_value, int_out_of_range,
string_chars, budget, shown_in_statement.

  budget  a case whose inputs are large enough that a natural Aval solution could exceed the interpreter's
          limits (10M steps per case, recursion depth 10,000): by default an Int argument above 100,000 in
          absolute value, or a String/List argument longer than 400 elements. The default is crude; the
          reference authors raise it per task in `mbpp_limits.json` (next to this script) when the natural
          solution is logarithmic or constant in the argument.

Statement = the MBPP prompt (URLs dropped, "python" removed before function/program/script/code, so that no
statement names a language), a newline, and `Example: name(args) returns value` for the first kept case whose
rendering does not contain "python", which is removed from `hidden` (`shown_in_statement`). `atol` as for HumanEval+: the call's atol; 1e-6 for results of
type Float / List<Float> when it is 0.
"""
import argparse
import ast
import hashlib
import json
import math
import os
import re
import sys
import warnings

warnings.filterwarnings("ignore", category=SyntaxWarning)    # the dataset's code contains invalid escapes

REVISION = "b2d74c91837c3f2a20c1299ae98133cbe7cfa077"
SHA256 = "dc20030b3788fccf617444edcb34138ef13d7e4fafd17bfcb8c1279dbb12399b"
MIN_CASES = 21          # kept cases before the example is taken out: at least 20 hidden cases remain
MAX_INT = 100000
MAX_LEN = 400

AVAL_KEYWORDS = set("""fn let return type enum module import pub if else match for in par requires ensures result
examples oracle effects alloc cost latency reuse none resource using unsafe true false and or not""".split())
RUST_KEYWORDS = set("""as break const continue crate else enum extern false fn for if impl in let loop match mod move mut
pub ref return self Self static struct super trait true type unsafe use where while async await dyn abstract become
box do final macro override priv typeof unsized virtual yield try main""".split())
PRELUDE = {"approx", "print"}


# ---- literal evaluation --------------------------------------------------------------------------------
class NotLiteral(Exception):
    pass


def lit(node):
    if isinstance(node, ast.Constant):
        return node.value
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, (ast.USub, ast.UAdd)):
        v = lit(node.operand)
        if isinstance(v, (int, float)) and not isinstance(v, bool):
            return -v if isinstance(node.op, ast.USub) else v
        raise NotLiteral()
    if isinstance(node, ast.List):
        return [lit(x) for x in node.elts]
    if isinstance(node, ast.Tuple):
        return tuple(lit(x) for x in node.elts)
    if isinstance(node, ast.Set):
        return set(lit(x) for x in node.elts)
    if isinstance(node, ast.Dict):
        return {("k", repr(lit(k))): lit(v) for k, v in zip(node.keys, node.values)}
    if isinstance(node, ast.Name) and node.id in ("inf", "nan"):
        return float(node.id)
    if (isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "float"
            and len(node.args) == 1 and isinstance(node.args[0], ast.Constant)):
        return float(node.args[0].value)
    raise NotLiteral()


def parse_line(test, name):
    m = re.search(r"^%s = (.*)$" % name, test, re.M)
    if not m:
        raise NotLiteral()
    return lit(ast.parse(m.group(1), mode="eval").body)


# ---- type inference -------------------------------------------------------------------------------------
class Unsupported(Exception):
    def __init__(self, reason):
        self.reason = reason


def shape(v):
    """Aval type of a Python value as a string; '?' for the element type of an empty list."""
    if isinstance(v, bool):
        return "Bool"
    if isinstance(v, int):
        return "Int"
    if isinstance(v, float):
        return "Float"
    if isinstance(v, str):
        return "String"
    if isinstance(v, list):
        inner = {shape(x) for x in v}
        inner = unify_set(inner)
        return "List<%s>" % inner
    raise Unsupported("unsupported_value")


def unify_set(shapes):
    shapes = {s for s in shapes if s != "?"}
    if not shapes:
        return "?"
    if len(shapes) == 1:
        return next(iter(shapes))
    lists = [s for s in shapes if s.startswith("List<")]
    if len(lists) == len(shapes):
        inner = unify_set({s[5:-1] for s in lists})
        return "List<%s>" % inner
    if shapes == {"Int", "Float"}:
        return "Float"
    raise Unsupported("type_mismatch")


def accepts(ty, v):
    """Does value v conform to Aval type ty (an Int is accepted for Float)?"""
    if ty == "Bool":
        return isinstance(v, bool)
    if ty == "Int":
        return isinstance(v, int) and not isinstance(v, bool)
    if ty == "Float":
        return isinstance(v, (int, float)) and not isinstance(v, bool)
    if ty == "String":
        return isinstance(v, str)
    if ty.startswith("List<"):
        return isinstance(v, list) and all(accepts(ty[5:-1], x) for x in v)
    return False


def promote(shapes):
    """Unify shapes where Int and Float occur in the same position: Float (ints are accepted as floats)."""
    shapes = {s for s in shapes if "?" not in s}
    if not shapes:
        return None

    def norm(s):
        return s.replace("Int", "Float")
    if len({norm(x) for x in shapes}) == 1:
        # same structure; Float wherever any variant has Float in that position
        if len(shapes) == 1:
            return next(iter(shapes))
        return norm(next(iter(shapes)))
    return None


def depth_of(shape_str):
    return shape_str.count("List<")


def norm_shape(shape_str):
    return shape_str.replace("Float", "Int")


def majority_type(values, allow_promote=True, orig=None):
    """Type of the values.

    `orig` is the shape of the corresponding argument in the first assertion of the original MBPP `test_list`
    (or None). When given, only shapes with the same list nesting depth are candidates, so the signature cannot be
    deeper or shallower than the original problem; ties are broken by that shape (modulo Int/Float), never
    alphabetically by accident. The type is the Int/Float-promoted type when all candidate cases agree up to
    Int/Float and every value has a supported structure, else the most frequent complete candidate."""
    shapes, all_supported = [], True
    for v in values:
        try:
            sh = shape(v)
        except Unsupported:
            all_supported = False
            continue
        if "?" in sh:
            continue
        shapes.append(sh)
    if orig is not None:
        shapes = [sh for sh in shapes if depth_of(sh) == depth_of(orig)]
    if not shapes:
        return None
    if allow_promote and all_supported:
        p = promote(set(shapes))
        if p:
            return p
    counts = {}
    for sh in shapes:
        counts[sh] = counts.get(sh, 0) + 1
    want = norm_shape(orig) if orig else None
    return sorted(counts.items(), key=lambda kv: (-kv[1], 0 if norm_shape(kv[0]) == want else 1, kv[0]))[0][0]


def orig_arg_shapes(row, entry):
    """Shapes of the arguments of the entry-point call in the first assertion of the original test_list."""
    try:
        tree = ast.parse(row["test_list"][0].strip())
    except (SyntaxError, IndexError, TypeError):
        return None
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == entry:
            out = []
            for a in node.args:
                try:
                    v = lit(a)
                    out.append(shape(as_lists(v)))
                except (NotLiteral, Unsupported, RecursionError):
                    out.append(None)
            return out
    return None


def as_lists(v):
    if isinstance(v, (list, tuple)):
        return [as_lists(x) for x in v]
    return v


def check_value(v, ty):
    """Return the normalised value or raise Unsupported."""
    if ty == "Bool":
        return v
    if ty == "Int":
        if not -2 ** 63 <= v < 2 ** 63:
            raise Unsupported("int_out_of_range")
        return v
    if ty == "Float":
        if isinstance(v, int) and abs(v) >= 2 ** 53:
            raise Unsupported("int_out_of_range")
        if not math.isfinite(v):
            raise Unsupported("unsupported_value")
        return float(v)
    if ty == "String":
        for c in v:
            if ord(c) > 126 or (ord(c) < 32 and c not in "\n\t"):
                raise Unsupported("string_chars")
        return v
    inner = ty[5:-1]
    out, reason = [], None
    for x in v:
        try:
            out.append(check_value(x, inner))
        except Unsupported as u:
            reason = reason or u.reason
    if reason:
        raise Unsupported(reason)
    return out


def too_big(v, max_int, max_len):
    if isinstance(v, bool):
        return False
    if isinstance(v, int):
        return abs(v) > max_int
    if isinstance(v, (str, list)):
        return len(v) > max_len or (isinstance(v, list) and any(too_big(x, max_int, max_len) for x in v))
    return False


# ---- names ------------------------------------------------------------------------------------------------
def snake(name):
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", s).lower()
    s = re.sub(r"[^a-z0-9_]", "_", s)
    if not re.match(r"[a-z_]", s):
        s = "_" + s
    return s


def safe(name, taken):
    s = snake(name)
    while s in AVAL_KEYWORDS or s in RUST_KEYWORDS or s in PRELUDE or s in taken:
        s += "_"
    taken.add(s)
    return s


def rust_type(ty):
    if ty.startswith("List<"):
        return "Vec<%s>" % rust_type(ty[5:-1])
    return {"Int": "i64", "Float": "f64", "Bool": "bool", "String": "String"}[ty]


# ---- rendering --------------------------------------------------------------------------------------------
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


def excluded_ids(path):
    ids = set()
    if path and os.path.exists(path):
        for line in open(path, encoding="utf-8"):
            m = re.match(r"\|\s*(MB-\d+)\s*\|", line)
            if m:
                ids.add(m.group(1))
    return ids


def neutral(prompt):
    """Statement text without the name of a programming language: URLs are dropped and `python` before
    function/program/script/code is removed ("Write a python function" -> "Write a function")."""
    t = re.sub(r"\s*https?://\S+", "", prompt.strip())
    t = re.sub(r"\b[Pp]ython\s+(?=(function|program|script|code)\b)", "", t)
    t = re.sub(r"[ \t]{2,}", " ", t).strip()
    return t


# ---- one task ----------------------------------------------------------------------------------------------
def convert(row, limits=None):
    """Return (task dict | None, skip reason | None)."""
    test = row["test"]
    call = re.search(r"^\s+assertion\((\w+)\(\*inp\), exp, ([0-9.e+-]+)\)\s*$", test, re.M)
    if not call:
        return None, "no_plain_assertion"
    if re.search(r"out = set\(out\)", test):
        return None, "unordered_result"
    if "def assertion(out, exp, atol):\n    if isinstance(out, bool)" in test:
        return None, "no_plain_assertion"
    entry, atol = call.group(1), float(call.group(2))
    try:
        inputs = parse_line(test, "inputs")
        results = parse_line(test, "results")
    except (NotLiteral, SyntaxError, RecursionError):
        return None, "unparsable"
    if len(inputs) != len(results):
        return None, "unparsable"
    # signature from the canonical solution
    params = None
    try:
        for node in ast.parse(row["code"]).body:
            if isinstance(node, ast.FunctionDef) and node.name == entry:
                params = [a.arg for a in node.args.args]
                if node.args.vararg or node.args.kwonlyargs or node.args.defaults:
                    params = None
    except SyntaxError:
        return None, "unparsable"
    if params is None:
        return None, "no_signature"
    if any(len(i) != len(params) for i in inputs) or not params:
        return None, "no_signature"
    lim = (limits or {}).get("MB-%03d" % row["task_id"], {})
    origs = orig_arg_shapes(row, entry)
    if origs is None or len(origs) != len(params):
        origs = [None] * len(params)

    def build(allow_promote):
        ptypes = []
        for k in range(len(params)):
            ty = majority_type([i[k] for i in inputs], allow_promote, origs[k])
            if ty is None:
                return None
            ptypes.append(ty)
        rtype = majority_type(results, allow_promote)
        if rtype is None:
            return None
        hidden, excluded, seen = [], {}, set()
        for i, e in zip(inputs, results):
            reason = None
            try:
                if not all(accepts(ty, v) for v, ty in zip(i, ptypes)) or not accepts(rtype, e):
                    for v in list(i) + [e]:
                        shape(v)            # raises unsupported_value for tuples, None, ...
                    raise Unsupported("type_mismatch")
                args_c = [check_value(v, ty) for v, ty in zip(i, ptypes)]
                exp_c = check_value(e, rtype)
            except Unsupported as u:
                reason = u.reason
            if reason is None and any(too_big(v, lim.get("max_int", MAX_INT), lim.get("max_len", MAX_LEN)) for v in i):
                reason = "budget"
            if reason:
                excluded[reason] = excluded.get(reason, 0) + 1
                continue
            key = json.dumps([args_c, exp_c])
            if key in seen:
                continue
            seen.add(key)
            hidden.append({"args": args_c, "expected": exp_c})
        return ptypes, rtype, hidden, excluded

    def conflicting(hidden):
        by_args = {}
        for h in hidden:
            by_args.setdefault(json.dumps(h["args"]), set()).add(json.dumps(h["expected"]))
        return any(len(v) > 1 for v in by_args.values())

    built = build(True)
    if built is None or conflicting(built[2]):
        # promoting Int to Float merged cases that Python distinguishes (e.g. isinstance checks): keep exact types
        built = build(False)
    if built is None:
        return None, "no_type"
    ptypes, rtype, hidden, excluded = built
    if len(hidden) < MIN_CASES:
        return None, "few_cases"
    taken = set()
    name = safe(entry, taken)
    pnames = [safe(p, taken) for p in params]
    pick = next((n for n, h in enumerate(hidden)
                 if "python" not in (json.dumps(h["args"]) + json.dumps(h["expected"])).lower()), 0)
    shown = hidden.pop(pick)
    excluded["shown_in_statement"] = 1
    example = "Example: %s(%s) returns %s" % (name, ", ".join(js(a) for a in shown["args"]), js(shown["expected"]))
    num = row["task_id"]
    id_ = "MB-%03d" % num
    if rtype in ("Float", "List<Float>") and atol == 0:
        atol = 1e-6
    t = {
        "id": id_,
        "source": "MBPP/%d (CC-BY-4.0, https://github.com/google-research/google-research/tree/master/mbpp); entry point %s" % (num, entry),
        "source_tests": ("MBPP+ (EvalPlus, Liu et al. 2023, Apache-2.0, https://github.com/evalplus/evalplus), MBPP/%d from "
                         "https://huggingface.co/datasets/evalplus/mbppplus revision %s" % (num, REVISION)),
        "statement": neutral(row["prompt"]) + "\n" + example,
        "aval": {"name": name, "params": [[p, ty] for p, ty in zip(pnames, ptypes)], "returns": rtype},
        "rust_signature": "fn %s(%s) -> %s" % (name, ", ".join("%s: %s" % (p, rust_type(ty)) for p, ty in zip(pnames, ptypes)), rust_type(rtype)),
        "atol": 0 if atol == 0 else atol,
        "hidden": hidden,
        "excluded": dict(sorted(excluded.items())),
        "reference": {"aval": "experiments/e2-in-context/reference-mbpp/%s.aval" % id_,
                      "rust": "experiments/e2-in-context/reference-mbpp/%s.rs" % id_},
    }
    return t, None


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("parquet")
    ap.add_argument("--out", default=os.path.join(here, "..", "tasks-mbpp"))
    ap.add_argument("--excluded", default=None)
    ap.add_argument("--limits", default=os.path.join(here, "mbpp_limits.json"),
                    help="per-task overrides of the budget rule: {\"MB-006\": {\"max_int\": 1000000000000, \"max_len\": 2000}}")
    ap.add_argument("--no-verify", action="store_true")
    ap.add_argument("--report", action="store_true", help="print the skip reason of every task not written")
    args = ap.parse_args()
    if not args.no_verify:
        digest = hashlib.sha256(open(args.parquet, "rb").read()).hexdigest()
        if digest != SHA256:
            raise SystemExit("%s has SHA-256 %s, expected %s (evalplus/mbppplus revision %s)" % (args.parquet, digest, SHA256, REVISION))
    try:
        import pyarrow.parquet as pq
    except ImportError:
        raise SystemExit("pyarrow is required (development only): pip install pyarrow")
    rows = sorted(pq.read_table(args.parquet).to_pylist(), key=lambda r: r["task_id"])
    out = args.out
    limits = json.load(open(args.limits)) if os.path.exists(args.limits) else {}
    os.makedirs(out, exist_ok=True)
    for stale in os.listdir(out):
        if re.fullmatch(r"MB-\d+\.json", stale):
            os.remove(os.path.join(out, stale))
    skip_ids = excluded_ids(args.excluded or os.path.join(out, "EXCLUDED.md"))
    written, skipped = 0, {}
    for row in rows:
        t, why = convert(row, limits)
        id_ = "MB-%03d" % row["task_id"]
        if t is not None and id_ in skip_ids:
            t, why = None, "excluded_md"
        if t is None:
            skipped[why] = skipped.get(why, 0) + 1
            if args.report:
                print("skip %s %s" % (id_, why))
            continue
        with open(os.path.join(out, id_ + ".json"), "w", encoding="utf-8", newline="\n") as f:
            f.write(render(t))
        written += 1
    print("written=%d skipped=%s" % (written, dict(sorted(skipped.items()))))


if __name__ == "__main__":
    main()
