"""Shared helpers for the aggregate_runs conformance tests (EVH-040..042). Not a test."""
import copy, json, os, re, subprocess, sys, tempfile
from fractions import Fraction
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import REPO, check, fail

TOOL = os.path.join(REPO, "experiments", "e2-in-context", "tools", "aggregate_runs.py")
SKILL = "ab8521ec3af4d0f1954e49cc180ba687f90a19f34cb649c95bf89dab3edeaad5"
OTHER_SKILL = "0" * 63 + "1"
IDS = ["MB-%03d" % i for i in range(1, 11)]


def _arm(solved_at):
    """solved_at: None (never solved), "error" (provider error) or the attempt number that solved the task."""
    if solved_at == "error":
        return {"attempts": [{"attempt": 1, "answer": "", "prompt": "p", "compile_ok": False, "diagnostics": 0,
                              "failed_cases": [], "fault_kinds": [], "hidden_total": 5, "outcome": "provider_error",
                              "reason": "provider_error", "tokens_in": 10, "tokens_out": 0, "tokens_system": 4,
                              "tokens_source": "tiktoken", "value_failures": 0}],
                "compile_at_1": False, "failed_cases": [], "outcome": "provider_error", "pass_at_1": False,
                "reason": "provider_error", "resource_failure": False, "solved": False, "solved_at": None}
    n = solved_at if solved_at else 3
    attempts = []
    for a in range(1, n + 1):
        ok = solved_at == a
        attempts.append({"attempt": a, "answer": "x", "prompt": "p", "compile_ok": True, "diagnostics": 0,
                         "failed_cases": [] if ok else [0], "fault_kinds": [], "hidden_total": 5,
                         "outcome": "solved" if ok else "failed", "reason": None if ok else "hidden_failed",
                         "tokens_in": 10, "tokens_out": 5, "tokens_system": 4, "tokens_source": "tiktoken",
                         "value_failures": 0 if ok else 1})
    return {"attempts": attempts, "compile_at_1": True, "failed_cases": [],
            "outcome": "solved" if solved_at else "failed", "pass_at_1": solved_at == 1,
            "reason": None if solved_at else "hidden_failed", "resource_failure": False,
            "solved": bool(solved_at), "solved_at": solved_at}


def make_results(aval_solved_at, rust_solved_at, ids=IDS, skill=SKILL, repairs=2, model="m-1",
                 provider="command", generated_at="2026-10-29T10:00:00Z", result=None):
    """aval_solved_at / rust_solved_at: dict task id -> None | attempt number that solved it."""
    tasks = {t: {"aval": _arm(aval_solved_at[t]), "rust": _arm(rust_solved_at[t])} for t in ids}
    metrics = {}
    for lang, sa in (("aval", aval_solved_at), ("rust", rust_solved_at)):
        n1 = sum(1 for t in ids if sa[t] == 1)
        metrics[lang] = {"tasks": len(ids), "compile_at_1": 1.0, "pass_at_1": n1 / len(ids), "pass_at_1_count": n1,
                         "pass_at_r": sum(1 for t in ids if sa[t] and sa[t] != "error") / len(ids),
                         "solved": sum(1 for t in ids if sa[t] and sa[t] != "error"), "provider_errors": 0, "resource_failures": 0,
                         "tokens_in": 1, "tokens_out": 1, "token_sources": ["tiktoken"], "unsolved_reasons": {}}
    # exact rational criterion (never floats): aval >= 4/5 * rust, from k/n
    n = len(ids)
    a = Fraction(metrics["aval"]["pass_at_1_count"], n)
    r = Fraction(metrics["rust"]["pass_at_1_count"], n)
    verdict = {"criterion": "Aval pass@1 >= 0.8 x Rust pass@1", "aval_pass_at_1": float(a), "rust_pass_at_1": float(r),
               "threshold": float(Fraction(4, 5) * r), "result": result or ("PASS" if a >= Fraction(4, 5) * r else "FAIL"), "degenerate": False}
    return {"schema": 1, "generated_at": generated_at, "languages": ["aval", "rust"], "metrics": metrics,
            "model": model, "provider": provider, "repairs": repairs, "skill_sha256": skill,
            "task_count": len(ids), "task_set": "MBPP+", "tasks": tasks, "tasks_done": len(ids),
            "verdict": verdict}


def ids_n(n):
    return ["MB-%03d" % i for i in range(1, n + 1)]


def solved_map(n_first_try, fail_ids=(), late=None, ids=IDS):
    """All tasks solved at attempt 1 except fail_ids; `late` maps task id -> attempt (or None for unsolved)."""
    m = {t: (None if t in fail_ids else 1) for t in ids}
    m.update(late or {})
    return m


def write_run(root, name, results, distinct=True):
    """Runs must be distinct files: by default the timestamp is made unique per run directory name."""
    d = os.path.join(root, name)
    os.makedirs(d)
    if distinct and isinstance(results, dict):
        results = dict(results, generated_at="2026-10-29T10:00:00Z#" + name)
    p = os.path.join(d, "results.json")
    json.dump(results, open(p, "w"), indent=1, sort_keys=True)
    return p


def aggregate(out, paths):
    check(os.path.isfile(TOOL), "aggregate_runs.py is missing: " + TOOL)
    return subprocess.run([sys.executable, TOOL, "--out", out] + list(paths), capture_output=True, text=True,
                          timeout=120, cwd=REPO)


def workdir():
    return tempfile.TemporaryDirectory(prefix="x")


def numbers(text):
    return [float(m) for m in re.findall(r"(?<![\w.])-?\d+(?:\.\d+)?(?:[eE]-?\d+)?", text)]


def has_value(nums, v, digits_tol=6e-4):
    """v (a fraction) appears as a fraction (+-6e-4) or as a percentage (+-6e-2)."""
    return any(abs(n - v) <= digits_tol or abs(n - 100 * v) <= digits_tol * 100 for n in nums)


def json_numbers(o):
    if isinstance(o, bool):
        return []
    if isinstance(o, (int, float)):
        return [float(o)]
    if isinstance(o, dict):
        return [x for v in o.values() for x in json_numbers(v)]
    if isinstance(o, list):
        return [x for v in o for x in json_numbers(v)]
    return []


def out_untouched(out):
    """True if --out does not exist or is empty (a rejected invocation writes nothing)."""
    return (not os.path.exists(out)) or (os.path.isdir(out) and not os.listdir(out))


def mean_line(md):
    m = re.findall(r"^Verdict on mean: (PASS|FAIL)\s*$", md, re.M)
    check(len(m) == 1, "expected exactly one line 'Verdict on mean: PASS|FAIL' in AGGREGATE.md:\n" + md)
    return m[0]


def every_line(md):
    m = re.findall(r"^Every run satisfies the criterion: (yes|no)\s*$", md, re.M)
    check(len(m) == 1, "expected exactly one line 'Every run satisfies the criterion: yes|no' in AGGREGATE.md:\n" + md)
    return m[0]


def load_out(out):
    md_path, js_path = os.path.join(out, "AGGREGATE.md"), os.path.join(out, "aggregate.json")
    check(os.path.isfile(md_path), "AGGREGATE.md not written to --out")
    check(os.path.isfile(js_path), "aggregate.json not written to --out")
    return open(md_path, encoding="utf-8").read(), json.load(open(js_path, encoding="utf-8"))


def close(a, b):
    return a is not None and b is not None and abs(a - b) <= 1e-9
