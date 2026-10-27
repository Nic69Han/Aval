#!/usr/bin/env python3
"""E2 harness: can a model that has never seen Aval write correct Aval from the in-context
reference (`aval skill print`) alone, compared with Rust on the same tasks?

Python 3 standard library only (tiktoken is used for token counts when installed).

Commands
    validate-tasks   check every task file against the schema (EVH-001)
    run              run tasks against a provider, write results.json and RESULTS.md

See README.md for the options and the file formats; the design is in
openspec/changes/add-e2-harness/design.md.
"""

import argparse
import binascii
import datetime
import hashlib
import json
import math
import os
import random
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from fractions import Fraction

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
DEFAULT_TASKS = os.path.join(HERE, "tasks")
LANGS = ("aval", "rust")
FENCE = {"aval": "aval", "rust": "rust"}
MAX_FEEDBACK_CHARS = 6000


class HarnessError(Exception):
    """Unrecoverable problem with the environment (missing binary, bad option)."""


# --------------------------------------------------------------------------------------
# Types and value conversion
# --------------------------------------------------------------------------------------

SCALARS = ("Int", "Float", "Bool", "String")


def parse_type(text):
    """Parse `Int | Float | Bool | String | List<T> | Result<T, E>` into a tuple tree."""
    pos = 0

    def ws():
        nonlocal pos
        while pos < len(text) and text[pos].isspace():
            pos += 1

    def ident():
        nonlocal pos
        ws()
        m = re.compile(r"[A-Za-z][A-Za-z0-9_]*").match(text, pos)
        if not m:
            raise ValueError("bad type %r" % text)
        pos = m.end()
        return m.group(0)

    def eat(ch):
        nonlocal pos
        ws()
        if pos >= len(text) or text[pos] != ch:
            raise ValueError("bad type %r (expected %r)" % (text, ch))
        pos += 1

    def ty():
        name = ident()
        if name in SCALARS:
            return (name,)
        if name == "List":
            eat("<")
            t = ty()
            eat(">")
            return ("List", t)
        if name == "Result":
            eat("<")
            a = ty()
            eat(",")
            b = ty()
            eat(">")
            return ("Result", a, b)
        raise ValueError("unsupported type %r" % name)

    t = ty()
    ws()
    if pos != len(text):
        raise ValueError("trailing text in type %r" % text)
    return t


def _float_text(x):
    x = float(x)
    if math.isnan(x) or math.isinf(x):
        raise ValueError("non-finite float")
    r = repr(x)
    if "e" in r or "E" in r:
        m, e = r.lower().split("e")
        if "." not in m:
            m += ".0"
        return "%se%d" % (m, int(e))
    if "." not in r:
        r += ".0"
    return r


def _check_value(v, t):
    """Raise ValueError unless JSON value v has type t."""
    k = t[0]
    if k == "Int":
        if not isinstance(v, int) or isinstance(v, bool):
            raise ValueError("expected Int, got %r" % (v,))
    elif k == "Float":
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise ValueError("expected Float, got %r" % (v,))
        _float_text(v)
    elif k == "Bool":
        if not isinstance(v, bool):
            raise ValueError("expected Bool, got %r" % (v,))
    elif k == "String":
        if not isinstance(v, str):
            raise ValueError("expected String, got %r" % (v,))
    elif k == "List":
        if not isinstance(v, list):
            raise ValueError("expected List, got %r" % (v,))
        for x in v:
            _check_value(x, t[1])
    elif k == "Result":
        if not (isinstance(v, dict) and len(v) == 1 and next(iter(v)) in ("Ok", "Err")):
            raise ValueError('expected {"Ok": v} or {"Err": e}, got %r' % (v,))
        which = next(iter(v))
        _check_value(v[which], t[1] if which == "Ok" else t[2])


def _aval_string(s):
    out = []
    for ch in s:
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\t":
            out.append("\\t")
        elif ord(ch) < 32 or ord(ch) == 127:
            raise ValueError("control character %r has no Aval escape" % ch)
        else:
            out.append(ch)
    return '"' + "".join(out) + '"'


def _rust_string(s):
    out = []
    for ch in s:
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\r":
            out.append("\\r")
        elif ch == "\t":
            out.append("\\t")
        elif ord(ch) < 32 or ord(ch) == 127:
            out.append("\\u{%x}" % ord(ch))
        else:
            out.append(ch)
    return '"' + "".join(out) + '"'


def aval_literal(v, t):
    k = t[0]
    if k == "Int":
        return str(v)
    if k == "Float":
        return _float_text(v)
    if k == "Bool":
        return "true" if v else "false"
    if k == "String":
        return _aval_string(v)
    if k == "List":
        return "[" + ", ".join(aval_literal(x, t[1]) for x in v) + "]"
    which = next(iter(v))
    return "%s(%s)" % (which, aval_literal(v[which], t[1] if which == "Ok" else t[2]))


def rust_literal(v, t, ref_str=False):
    """Rust expression for v. With ref_str a String is emitted as a `&str` literal."""
    k = t[0]
    if k == "Int":
        return str(v)
    if k == "Float":
        return _float_text(v)
    if k == "Bool":
        return "true" if v else "false"
    if k == "String":
        lit = _rust_string(v)
        return lit if ref_str else "String::from(%s)" % lit
    if k == "List":
        return "vec![" + ", ".join(rust_literal(x, t[1]) for x in v) + "]"
    which = next(iter(v))
    return "%s(%s)" % (which, rust_literal(v[which], t[1] if which == "Ok" else t[2]))


class _Names:
    def __init__(self):
        self.n = 0

    def fresh(self):
        self.n += 1
        return "hv_%d" % self.n


# EvalPlus: abs(got - expected) <= atol + rtol * abs(expected), rtol = 1e-7 (when atol > 0)
RTOL = 1e-7


def _paren_if_neg(text):
    return "(%s)" % text if text.startswith("-") else text


def aval_check_expr(path, v, t, names, atol=0.0):
    """Boolean Aval expression: does `path` equal v? Floats: `abs(got - exp) <= atol` when
    atol > 0 (EvalPlus semantics), exact equality otherwise."""
    k = t[0]
    if k == "Float":
        if atol > 0:
            tol = _float_text(atol + RTOL * abs(float(v)))
            return "((%s - %s).abs() <= %s)" % (path, _paren_if_neg(_float_text(v)), tol)
        return "(%s == %s)" % (path, _float_text(v))
    if k in ("Int", "Bool", "String"):
        return "(%s == %s)" % (path, aval_literal(v, t))
    if k == "List":
        parts = ["%s.len() == %d" % (path, len(v))]
        for i, x in enumerate(v):
            parts.append(aval_check_expr("%s[%d]" % (path, i), x, t[1], names, atol))
        return "(" + " and ".join(parts) + ")"
    which = next(iter(v))
    other = "Err" if which == "Ok" else "Ok"
    name = names.fresh()
    inner = aval_check_expr(name, v[which], t[1] if which == "Ok" else t[2], names, atol)
    return "match %s {\n    %s(%s) => %s\n    %s(_) => false\n  }" % (
        path, which, name, inner, other)


def rust_check_expr(path, v, t, names, atol=0.0):
    k = t[0]
    if k == "Float":
        if atol > 0:
            tol = _float_text(atol + RTOL * abs(float(v)))
            return "((%s - (%s)).abs() <= %s)" % (path, _float_text(v), tol)
        return "(%s == %s)" % (path, _float_text(v))
    if k == "Int":
        return "(%s == %s)" % (path, str(v))
    if k == "Bool":
        return "(%s == %s)" % (path, "true" if v else "false")
    if k == "String":
        return "(&*%s == %s)" % (path, _rust_string(v))
    if k == "List":
        parts = ["%s.len() == %d" % (path, len(v))]
        for i, x in enumerate(v):
            parts.append(rust_check_expr("%s[%d]" % (path, i), x, t[1], names, atol))
        return "(" + " && ".join(parts) + ")"
    which = next(iter(v))
    other = "Err" if which == "Ok" else "Ok"
    name = names.fresh()
    inner = rust_check_expr(name, v[which], t[1] if which == "Ok" else t[2], names, atol)
    return "match %s.clone() { %s(%s) => %s, %s(_) => false }" % (
        path, which, name, inner, other)


# --------------------------------------------------------------------------------------
# Tasks
# --------------------------------------------------------------------------------------

IDENT_RE = re.compile(r"^[a-z_][a-z0-9_]*$")
ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.-]*$")


class Task:
    def __init__(self, data, path):
        self.data = data
        self.path = path
        self.id = data["id"]
        self.statement = data["statement"]
        self.name = data["aval"]["name"]
        self.params = [(p[0], p[1]) for p in data["aval"]["params"]]
        self.ptypes = [parse_type(p[1]) for p in data["aval"]["params"]]
        self.returns = data["aval"]["returns"]
        self.rtype = parse_type(self.returns)
        self.rust_signature = data["rust_signature"]
        self.hidden = data["hidden"]
        self.atol = float(data.get("atol", 0) or 0)

    def aval_signature(self):
        ps = ", ".join("%s: %s" % (n, t) for n, t in self.params)
        return "fn %s(%s) -> %s" % (self.name, ps, self.returns)

    def reference_path(self, lang):
        return resolve_path(self.data["reference"][lang], self.path)


def resolve_path(p, task_path):
    """Reference paths are relative to the repository root; fall back to the task's own tree."""
    if os.path.isabs(p):
        return p
    cands = [os.path.join(REPO, p)]
    tdir = os.path.dirname(os.path.abspath(task_path))
    # <exp>/tasks/X.json with a reference path such as experiments/e2-in-context/reference/X.aval
    cands.append(os.path.join(os.path.dirname(tdir), "reference", os.path.basename(p)))
    cands.append(os.path.join(tdir, p))
    for c in cands:
        if os.path.exists(c):
            return c
    return cands[0]


def split_top_level(s, sep=","):
    out, depth, cur = [], 0, []
    for ch in s:
        if ch in "<([{":
            depth += 1
        elif ch in ">)]}":
            depth -= 1
        if ch == sep and depth == 0:
            out.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    tail = "".join(cur)
    if tail.strip():
        out.append(tail)
    return out


def rust_param_types(sig):
    """Parameter types of a Rust signature (text after `name:`), in order."""
    m = re.search(r"\bfn\s+[A-Za-z_][A-Za-z0-9_]*\s*(?:<[^>]*>)?\s*\(", sig)
    if not m:
        raise ValueError("cannot find parameter list")
    start = m.end()
    depth, i = 1, start
    while i < len(sig) and depth:
        if sig[i] == "(":
            depth += 1
        elif sig[i] == ")":
            depth -= 1
        i += 1
    if depth:
        raise ValueError("unbalanced parentheses")
    inner = sig[start:i - 1]
    types = []
    for part in split_top_level(inner):
        if ":" not in part:
            raise ValueError("parameter without a type: %r" % part.strip())
        types.append(part.split(":", 1)[1].strip())
    return types


def validate_task(data, path):
    """Return a list of error strings (empty when the task is valid)."""
    errs = []

    def need(key, kind, where=data, label=None):
        label = label or key
        if key not in where:
            errs.append("missing %s" % label)
            return None
        if not isinstance(where[key], kind):
            errs.append("%s has the wrong type" % label)
            return None
        return where[key]

    tid = need("id", str)
    if tid is not None:
        if not ID_RE.match(tid):
            errs.append("bad id %r" % tid)
        elif os.path.splitext(os.path.basename(path))[0] != tid:
            errs.append("file name does not match id %r" % tid)
    src = need("source", str)
    if src is not None and not src.strip():
        errs.append("empty source")
    st = need("statement", str)
    if st is not None:
        if not st.strip():
            errs.append("empty statement")
        if "```" in st:
            errs.append("statement must not contain code fences")
    av = need("aval", dict)
    ptypes, rtype, name = [], None, None
    if av is not None:
        name = need("name", str, av, "aval.name")
        if name is not None and not IDENT_RE.match(name):
            errs.append("aval.name %r is not a snake_case identifier" % name)
        params = need("params", list, av, "aval.params")
        if params is not None:
            seen = set()
            for p in params:
                if not (isinstance(p, list) and len(p) == 2 and all(isinstance(x, str) for x in p)):
                    errs.append("aval.params entries must be [name, type]")
                    continue
                if not IDENT_RE.match(p[0]) or p[0] in seen:
                    errs.append("bad or duplicate parameter name %r" % p[0])
                seen.add(p[0])
                try:
                    ptypes.append(parse_type(p[1]))
                except ValueError as e:
                    errs.append("aval.params: %s" % e)
                    ptypes.append(None)
        ret = need("returns", str, av, "aval.returns")
        if ret is not None:
            try:
                rtype = parse_type(ret)
            except ValueError as e:
                errs.append("aval.returns: %s" % e)
    rs = need("rust_signature", str)
    if rs is not None:
        if not re.search(r"^\s*(pub\s+)?fn\s+", rs):
            errs.append("rust_signature must start with `fn`")
        elif name and not re.search(r"\bfn\s+%s\s*[(<]" % re.escape(name), rs):
            errs.append("rust_signature does not declare fn %s" % name)
        else:
            try:
                rp = rust_param_types(rs)
                if len(rp) != len(ptypes):
                    errs.append("rust_signature has %d parameters, aval has %d" % (len(rp), len(ptypes)))
                if any(t.startswith("&mut") for t in rp):
                    errs.append("rust_signature: &mut parameters are not supported")
            except ValueError as e:
                errs.append("rust_signature: %s" % e)
    hidden = need("hidden", list)
    if hidden is not None:
        if len(hidden) < 20:
            errs.append("needs at least 20 hidden cases, has %d" % len(hidden))
        for i, h in enumerate(hidden):
            if not (isinstance(h, dict) and "args" in h and "expected" in h):
                errs.append("hidden[%d] needs args and expected" % i)
                continue
            if not isinstance(h["args"], list) or len(h["args"]) != len(ptypes):
                errs.append("hidden[%d].args has the wrong length" % i)
                continue
            try:
                for a, t in zip(h["args"], ptypes):
                    if t is not None:
                        _check_value(a, t)
                        aval_literal(a, t)
                        rust_literal(a, t)
                if rtype is not None:
                    _check_value(h["expected"], rtype)
                    aval_literal(h["expected"], rtype)
                    rust_literal(h["expected"], rtype)
            except ValueError as e:
                errs.append("hidden[%d]: %s" % (i, e))
    if "atol" in data:
        a = data["atol"]
        if isinstance(a, bool) or not isinstance(a, (int, float)) or a < 0 or a != a or a == float("inf"):
            errs.append("atol must be a non-negative number")
    st = need("source_tests", str)
    if st is not None and "HumanEval+" not in st and "MBPP+" not in st:
        errs.append("source_tests must mention HumanEval+ or MBPP+")
    need("excluded", dict)
    ref = need("reference", dict)
    if ref is not None:
        for lang in LANGS:
            p = need(lang, str, ref, "reference.%s" % lang)
            if p is None:
                continue
            full = resolve_path(p, path)
            if not os.path.isfile(full):
                errs.append("reference.%s: file %s not found" % (lang, p))
                continue
            with open(full, encoding="utf-8") as f:
                text = f.read()
            if extract_code(text, lang) is None:
                errs.append("reference.%s has no ```%s block" % (lang, FENCE[lang]))
    return errs


def load_tasks(tasks_dir, only=None):
    """Return (tasks, errors). Tasks are sorted by id; errors map task id/file to messages."""
    if not os.path.isdir(tasks_dir):
        raise HarnessError("tasks directory not found: %s" % tasks_dir)
    files = sorted(f for f in os.listdir(tasks_dir) if f.endswith(".json"))
    tasks, errors, seen = [], {}, set()
    for f in files:
        path = os.path.join(tasks_dir, f)
        label = os.path.splitext(f)[0]
        try:
            with open(path, encoding="utf-8") as fh:
                data = json.load(fh)
        except (OSError, ValueError) as e:
            errors[label] = ["cannot read: %s" % e]
            continue
        if not isinstance(data, dict):
            errors[label] = ["task file must hold a JSON object"]
            continue
        errs = validate_task(data, path)
        if not errs and data["id"] in seen:
            errs.append("duplicate id")
        if errs:
            errors[label] = errs
            continue
        seen.add(data["id"])
        tasks.append(Task(data, path))
    tasks.sort(key=lambda t: t.id)
    if only:
        wanted = set(only)
        unknown = wanted - {t.id for t in tasks}
        if unknown:
            raise HarnessError("unknown or invalid task id(s): %s" % ", ".join(sorted(unknown)))
        tasks = [t for t in tasks if t.id in wanted]
    return tasks, errors


def cmd_validate(args):
    tasks, errors = load_tasks(args.tasks)
    for label in sorted(errors):
        for e in errors[label]:
            print("INVALID %s: %s" % (label, e))
    if errors:
        print("%d valid tasks, %d invalid" % (len(tasks), len(errors)))
        return 1
    if not tasks:
        print("0 valid tasks (no task files found)")
        return 1
    print("%d valid tasks" % len(tasks))
    return 0


# --------------------------------------------------------------------------------------
# Prompts and code extraction
# --------------------------------------------------------------------------------------

FENCE_RE = {
    lang: re.compile(r"```[ \t]*%s[ \t]*\r?\n(.*?)```" % FENCE[lang], re.DOTALL | re.IGNORECASE)
    for lang in LANGS
}


def extract_code(text, lang):
    m = FENCE_RE[lang].search(text)
    return m.group(1) if m else None


RUST_SYSTEM = (
    "You are an expert Rust programmer. Answer with the complete function in a single ```rust "
    "code block and nothing else that needs to compile."
)


def first_prompt(task, lang):
    if lang == "aval":
        return (
            "%s\n\nRequired signature:\n\n```\n%s\n```\n\n"
            "Write one `pub fn %s` with exactly this signature, with at least one `ensures` or "
            "`examples` clause and a `cost` clause. Start the file with `module solution`. "
            "Reply with the complete module in a single ```aval block."
        ) % (task.statement.strip(), task.aval_signature(), task.name)
    return (
        "%s\n\nRequired signature:\n\n```\n%s\n```\n\n"
        "Write the function `%s` with exactly this signature (std only, no `main`, helper items "
        "allowed). Reply with the code in a single ```rust block."
    ) % (task.statement.strip(), task.rust_signature.strip(), task.name)


def truncate(text):
    if len(text) <= MAX_FEEDBACK_CHARS:
        return text
    return text[:MAX_FEEDBACK_CHARS] + "\n... (truncated)"


def repair_prompt(lang, failure):
    kind = failure["kind"]
    fence = FENCE[lang]
    if kind == "no_code":
        return (
            "Your reply did not contain a ```%s code block. Reply with the complete solution "
            "in a single ```%s block." % (fence, fence)
        )
    if kind == "compile":
        if lang == "aval":
            head = (
                "`aval check` rejected the module. Diagnostics (one JSON object per line):\n\n%s"
                % truncate(failure["text"])
            )
        else:
            head = "`rustc` rejected the code:\n\n%s" % truncate(failure["text"])
    elif kind == "signature":
        head = (
            "The module compiles, but it could not be called with the required signature "
            "`%s`. Keep the name and parameter types exactly as required%s."
            % (failure["signature"], "" if lang == "aval" else " and do not define `main`")
        )
        if failure.get("problems"):
            head += "\nCompiler messages: " + "; ".join(failure["problems"])
    elif kind == "crash":
        head = ("Running the solution crashed (abnormal exit, abort or out of memory). "
                "Avoid unbounded recursion, unbounded growth and early process exit.")
    elif kind == "timeout":
        head = "Running the hidden tests timed out: the solution is too slow or does not terminate."
    else:
        head = ""
        if failure.get("text"):
            head = "Diagnostics from your own spec (one JSON object per line):\n\n%s\n\n" % truncate(
                failure["text"])
        head += "Hidden tests: %d of %d cases failed." % (failure["failed"], failure["total"])
    return (
        "%s\n\nFix the problem and reply with the complete corrected %s in a single ```%s block."
        % (head, "module" if lang == "aval" else "code", fence)
    )


# --------------------------------------------------------------------------------------
# Token accounting
# --------------------------------------------------------------------------------------


class Tokenizer:
    def __init__(self, mode="auto"):
        self.enc = None
        self.name = "estimate"
        if mode in ("auto", "tiktoken"):
            try:
                import tiktoken  # noqa: PLC0415

                self.enc = tiktoken.get_encoding("o200k_base")
                self.name = "tiktoken"
            except Exception as e:  # ImportError, or the vocabulary cannot be loaded offline
                if mode == "tiktoken":
                    raise HarnessError("tiktoken unavailable: %s" % e)

    def count(self, text):
        if self.enc is not None:
            return len(self.enc.encode(text, disallowed_special=()))
        return int(math.ceil(len(text) / 4.0))


# --------------------------------------------------------------------------------------
# Providers
# --------------------------------------------------------------------------------------


class ProviderError(Exception):
    pass


class Reply:
    def __init__(self, text, usage=None):
        self.text = text
        self.usage = usage  # (tokens_in, tokens_out) reported by the provider, or None


def flatten(system, messages):
    parts = ["[system]\n" + system]
    for m in messages:
        parts.append("[%s]\n%s" % (m["role"], m["content"]))
    return "\n\n".join(parts) + "\n"


class ReplayProvider:
    name = "replay"

    def __init__(self, responses, tasks):
        self.reference = responses == "reference"
        self.tasks = {t.id: t for t in tasks}
        self.data = None
        if not self.reference:
            try:
                with open(responses, encoding="utf-8") as f:
                    self.data = json.load(f)
            except (OSError, ValueError) as e:
                raise HarnessError("cannot read responses file: %s" % e)
            if not isinstance(self.data, dict):
                raise HarnessError("responses file must hold a JSON object")

    def complete(self, task, lang, attempt, system, messages):
        if self.reference:
            path = self.tasks[task.id].reference_path(lang)
            with open(path, encoding="utf-8") as f:
                return Reply(f.read())
        entry = self.data.get(task.id, {}).get(lang)
        if not entry:
            raise ProviderError("no_response")
        if isinstance(entry, str):
            entry = [entry]
        # attempts beyond the recorded list repeat the last recorded answer
        return Reply(entry[min(attempt, len(entry)) - 1])


class CommandProvider:
    name = "command"

    def __init__(self, command, timeout, pass_env=()):
        if not command:
            raise HarnessError("--provider command needs --command")
        self.command = command
        self.timeout = timeout
        self.pass_env = list(pass_env)

    def complete(self, task, lang, attempt, system, messages):
        # Isolation: an empty working directory and a minimal environment, so that an agentic
        # CLI cannot read the task files (hidden cases) or the reference solutions.
        env = {k: os.environ[k] for k in ("PATH", "HOME", "LANG") if k in os.environ}
        for k, v in os.environ.items():
            if k.startswith("AVAL_") or k in self.pass_env:
                env[k] = v
        env.update({"AVAL_TASK_ID": task.id, "AVAL_LANG": lang, "AVAL_ATTEMPT": str(attempt)})
        try:
            with tempfile.TemporaryDirectory(prefix="e2-cmd-") as cwd:
                r = subprocess.run(
                    self.command, shell=True, input=flatten(system, messages), cwd=cwd,
                    capture_output=True, text=True, env=env, timeout=self.timeout,
                )
        except subprocess.TimeoutExpired:
            raise ProviderError("provider_timeout")
        except OSError:
            raise ProviderError("provider_error")
        if r.returncode != 0:
            raise ProviderError("provider_error")
        return Reply(r.stdout)


class AnthropicProvider:
    name = "anthropic"
    URL = "https://api.anthropic.com/v1/messages"

    def __init__(self, model, max_tokens, timeout):
        self.key = os.environ.get("ANTHROPIC_API_KEY")
        if not self.key:
            raise HarnessError("ANTHROPIC_API_KEY is not set")
        if not model:
            raise HarnessError("--provider anthropic needs --model")
        self.model = model
        self.max_tokens = max_tokens
        self.timeout = timeout

    def complete(self, task, lang, attempt, system, messages):
        body = json.dumps({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "temperature": 0,
            "system": system,
            "messages": messages,
        }).encode("utf-8")
        headers = {
            "content-type": "application/json",
            "x-api-key": self.key,
            "anthropic-version": "2023-06-01",
        }
        last = "provider_error"
        for tries in range(4):
            req = urllib.request.Request(self.URL, data=body, headers=headers, method="POST")
            try:
                with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                    out = json.loads(resp.read().decode("utf-8"))
            except urllib.error.HTTPError as e:
                last = "provider_http_%d" % e.code
                if e.code in (429, 500, 502, 503, 529) and tries < 3:
                    time.sleep(2 ** tries * 2)
                    continue
                raise ProviderError(last)
            except (urllib.error.URLError, OSError, ValueError):
                last = "provider_error"
                if tries < 3:
                    time.sleep(2 ** tries * 2)
                    continue
                raise ProviderError(last)
            text = "".join(b.get("text", "") for b in out.get("content", []) if b.get("type") == "text")
            u = out.get("usage") or {}
            usage = None
            if "input_tokens" in u and "output_tokens" in u:
                usage = (int(u["input_tokens"]), int(u["output_tokens"]))
            return Reply(text, usage)
        raise ProviderError(last)


# --------------------------------------------------------------------------------------
# Running candidates
# --------------------------------------------------------------------------------------


def find_aval(explicit):
    if explicit:
        return shlex.split(explicit)
    env = os.environ.get("AVAL_BIN")
    if env:
        return shlex.split(env)
    local = os.path.join(REPO, "target", "debug", "aval")
    if os.path.isfile(local):
        return [local]
    found = shutil.which("aval")
    if found:
        return [found]
    if shutil.which("cargo"):
        return ["cargo", "run", "-q", "--manifest-path", os.path.join(REPO, "Cargo.toml"),
                "-p", "aval-cli", "--"]
    raise HarnessError("cannot find the aval binary: pass --aval-bin or set AVAL_BIN")


def run(cmd, cwd, timeout, stdin=None):
    """Run a command; returns (returncode or None on timeout, stdout, stderr)."""
    try:
        r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout,
                           input=stdin, errors="replace")
    except subprocess.TimeoutExpired:
        return None, "", ""
    except OSError as e:
        raise HarnessError("cannot run %s: %s" % (cmd[0], e))
    return r.returncode, r.stdout, r.stderr


def parse_diags(stdout):
    out = []
    for line in stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            d = json.loads(line)
        except ValueError:
            continue
        if isinstance(d, dict):
            out.append((line, d))
    return out


def is_error(d):
    return str(d.get("code", ""))[:1] in ("E", "T")


BATCH = 50
DEFAULT_STEPS = 10_000_000  # `aval test` per-input step budget
RESOURCE_FAULTS = ("step_limit", "memory_limit", "stack_limit")


def fault_kind(name):
    return name if name in RESOURCE_FAULTS + ("overflow",) else "other"


def batches(n):
    return [list(range(i, min(i + BATCH, n))) for i in range(0, n, BATCH)]


def _aval_case(task, i, names):
    h = task.hidden[i]
    args = ", ".join(aval_literal(a, t) for a, t in zip(h["args"], task.ptypes))
    check = aval_check_expr("r_%d" % i, h["expected"], task.rtype, names, task.atol)
    return ("  let r_%d: %s = %s(%s)\n  let c_%d: Bool = %s\n"
            % (i, task.returns, task.name, args, i, check))


def hidden_aval_batches(task):
    """One function per batch of up to 50 cases; it returns the index of the first failing
    case, or -1, and the `examples => -1` line makes `aval test` report T100 otherwise."""
    out = []
    for b, idxs in enumerate(batches(len(task.hidden))):
        names = _Names()
        body = "".join(_aval_case(task, i, names) for i in idxs)
        chain = "-1"
        for i in reversed(idxs):
            chain = "if c_%d { %s } else { %d }" % (i, chain, i)
        out.append("fn hidden_b%d() -> Int\n  examples\n    => -1\n{\n%s  return %s\n}\n"
                   % (b, body, chain))
    return "\n".join(out)


def hidden_aval_cases(task, which):
    """One function per case (used to locate the failures inside a failing batch)."""
    out = []
    for i in which:
        names = _Names()
        out.append("fn hidden_%d() -> Bool\n  examples\n    => true\n{\n%s  return c_%d\n}\n"
                   % (i, _aval_case(task, i, names), i))
    return "\n".join(out)


def hidden_rust(task, nonce):
    ptys = rust_param_types(task.rust_signature)
    n = len(task.hidden)
    lines = ["fn main() {", "    std::panic::set_hook(Box::new(|_| {}));",
             "    let cases: [fn() -> bool; %d] = [" % n]
    for i, h in enumerate(task.hidden):
        names = _Names()
        args = []
        for a, t, rt in zip(h["args"], task.ptypes, ptys):
            rt = rt.replace(" ", "")
            if rt.startswith("&str"):
                args.append(rust_literal(a, t, ref_str=True))
            elif rt.startswith("&"):
                args.append("&" + rust_literal(a, t))
            else:
                args.append(rust_literal(a, t))
        check = rust_check_expr("r", h["expected"], task.rtype, names, task.atol)
        lines.append("        || { let r = %s(%s); %s }," % (task.name, ", ".join(args), check))
    lines += [
        "    ];",
        "    for (b, chunk) in cases.chunks(%d).enumerate() {" % BATCH,
        "        let mut bits = String::new();",
        "        for c in chunk {",
        "            let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c())).unwrap_or(false);",
        "            bits.push(if ok { '1' } else { '0' });",
        "        }",
        '        println!("HIDDEN_BATCH {} %s {}", b, bits);' % nonce,
        "    }",
        '    println!("HIDDEN_DONE %s");' % nonce,
        "}",
    ]
    return "\n".join(lines) + "\n"


def _fail(compile_ok, total, reason, failure, diagnostics=0, failed=None):
    return {"compile_ok": compile_ok, "failed": list(range(total)) if failed is None else failed,
            "diagnostics": diagnostics, "failure": failure, "reason": reason}


def evaluate_aval(task, code, aval, timeout):
    """Returns a dict: compile_ok, failed (list of case indices), diagnostics (int), failure."""
    total = len(task.hidden)
    crash = {"kind": "crash"}
    with tempfile.TemporaryDirectory(prefix="e2-aval-") as d:
        path = os.path.join(d, "main.aval")
        with open(path, "w", encoding="utf-8") as f:
            f.write(code)
        rc, out, err = run(aval + ["check", "main.aval"], d, timeout)
        if rc is None:
            return _fail(False, total, "timeout", {"kind": "timeout"})
        if rc not in (0, 1):
            return _fail(False, total, "crash", crash)
        diags = parse_diags(out)
        if rc != 0 or any(is_error(x) for _, x in diags):
            return _fail(False, total, "compile_error",
                         {"kind": "compile", "text": "\n".join(l for l, _ in diags)}, len(diags))
        def test_with(extra, max_steps=None):
            with open(path, "w", encoding="utf-8") as f:
                f.write(code.rstrip("\n") + "\n\n" + extra)
            extra_args = ["--max-steps", str(max_steps)] if max_steps else []
            rc, out, err = run(aval + ["test", "main.aval"] + extra_args, d, timeout)
            if rc is None:
                return "timeout", None
            if rc not in (0, 1):
                return "crash", None
            return rc, parse_diags(out)

        # A batch of k cases gets k times the per-input step budget, so batching never gives a
        # case less budget than it would have alone.
        rc, diags = test_with(hidden_aval_batches(task), BATCH * DEFAULT_STEPS)
        if rc in ("timeout", "crash"):
            return _fail(True, total, rc, {"kind": rc})
        failed, own, problems, bad_batches = set(), [], [], set()
        kinds, value_failures = set(), 0
        all_batches = batches(total)
        for line, x in diags:
            if not is_error(x):
                continue
            code_ = str(x.get("code", ""))
            mb = re.match(r"hidden_b(\d+)$", str(x.get("fn", "")))
            if mb:
                if code_[:1] == "E":
                    problems.append("%s: %s" % (code_, x.get("msg", "")))
                elif int(mb.group(1)) < len(all_batches):
                    bad_batches.add(int(mb.group(1)))
            elif code_[:1] == "E":
                # an error that `aval check` of the candidate alone did not report
                problems.append("%s: %s" % (code_, x.get("msg", "")))
                failed.update(range(total))
            else:
                own.append(line)
        if problems:
            failed.update(range(total))
        elif bad_batches:
            # locate the failing cases inside the failing batches, one function per case
            which = [i for b in sorted(bad_batches) for i in all_batches[b]]
            rc2, diags2 = test_with(hidden_aval_cases(task, which))
            if rc2 in ("timeout", "crash"):
                return _fail(True, total, rc2, {"kind": rc2})
            for line, x in diags2:
                m = re.match(r"hidden_(\d+)$", str(x.get("fn", "")))
                if m and is_error(x):
                    failed.add(int(m.group(1)))
                    if x.get("code") == "T700":
                        kinds.add(fault_kind(str(x.get("fault", ""))))
                    else:
                        value_failures += 1
            if not failed:
                for b in bad_batches:
                    failed.update(all_batches[b])
        # Positive attestation: an exit code 1 must be explained by a diagnostic.
        if rc == 1 and not any(is_error(x) for _, x in diags):
            failed.update(range(total))
            problems.append("aval test failed without a diagnostic")
        failed = sorted(failed)
        ndiag = len([1 for _, x in diags if is_error(x) and not str(x.get("fn", "")).startswith("hidden_")])
        res = {"compile_ok": True, "failed": failed, "diagnostics": ndiag,
               "fault_kinds": sorted(kinds), "value_failures": value_failures}
        if failed:
            res["reason"] = "hidden_failed"
            if problems:
                res["failure"] = {"kind": "signature", "signature": task.aval_signature(),
                                  "problems": sorted(set(problems))}
            else:
                res["failure"] = {"kind": "hidden", "failed": len(failed), "total": total,
                                  "text": "\n".join(own)}
        return res


def scrub(text, d):
    return text.replace(d, "<dir>")


def evaluate_rust(task, code, timeout):
    total = len(task.hidden)
    rustc = shutil.which("rustc")
    if not rustc:
        raise HarnessError("rustc not found on PATH (needed for the Rust arm)")
    flags = ["--edition", "2021", "-A", "warnings"]
    crash = {"kind": "crash"}
    nonce = binascii.hexlify(os.urandom(8)).decode()
    with tempfile.TemporaryDirectory(prefix="e2-rust-") as d:
        lib = os.path.join(d, "cand.rs")
        with open(lib, "w", encoding="utf-8") as f:
            f.write(code)
        rc, out, err = run([rustc] + flags + ["--crate-type", "lib", "--emit", "metadata",
                                              "-o", os.path.join(d, "cand.rmeta"), "cand.rs"], d, timeout)
        if rc is None:
            return _fail(False, total, "timeout", {"kind": "timeout"})
        if rc not in (0, 1) or "internal compiler error" in err:
            return _fail(False, total, "crash", crash)
        if rc != 0:
            text = scrub(err, d).strip()
            n = len(re.findall(r"^error", text, re.MULTILINE))
            return _fail(False, total, "compile_error", {"kind": "compile", "text": text}, n)
        main = os.path.join(d, "main.rs")
        with open(main, "w", encoding="utf-8") as f:
            f.write(code.rstrip("\n") + "\n\n" + hidden_rust(task, nonce))
        exe = os.path.join(d, "main")
        rc, out, err = run([rustc] + flags + ["-O", "-C", "overflow-checks=on", "-o", exe, "main.rs"],
                           d, timeout)
        if rc is None:
            return _fail(True, total, "timeout", {"kind": "timeout"})
        if rc not in (0, 1) or "internal compiler error" in err:
            return _fail(True, total, "crash", crash)
        if rc != 0:
            # The error text would quote the harness's own source; report the signature instead.
            return _fail(True, total, "hidden_failed",
                         {"kind": "signature", "signature": task.rust_signature.strip()})
        # Run from a fresh empty directory with the sources removed, so the candidate cannot read
        # the generated main.rs. This guards against accidental early exit and stray output; it is
        # not a sandbox against a malicious candidate.
        for junk in ("cand.rs", "main.rs", "cand.rmeta"):
            try:
                os.remove(os.path.join(d, junk))
            except OSError:
                pass
        with tempfile.TemporaryDirectory(prefix="e2-run-") as cwd:
            rc, out, err = run([exe], cwd, timeout)
        if rc is None:
            return _fail(True, total, "timeout", {"kind": "timeout"})
        ok = set()
        for b_, bits in re.findall(r"^HIDDEN_BATCH (\d+) %s ([01]+)$" % nonce, out, re.MULTILINE):
            base = int(b_) * BATCH
            ok.update(base + k for k, ch in enumerate(bits) if ch == "1" and base + k < total)
        done = re.search(r"^HIDDEN_DONE %s$" % nonce, out, re.MULTILINE) is not None
        if rc != 0:
            # a panic outside the hidden cases, abort, signal, or a non-zero `exit`
            res = _fail(True, total, "crash", crash)
            res["failed"] = [i for i in range(total) if i not in ok]
            return res
        failed = [i for i in range(total) if i not in ok]
        if not done and not failed:
            failed = list(range(total))
        res = {"compile_ok": True, "failed": failed, "diagnostics": 0}
        if failed:
            res["reason"] = "hidden_failed"
            res["failure"] = {"kind": "hidden", "failed": len(failed), "total": total, "text": ""}
        return res


# --------------------------------------------------------------------------------------
# The attempt loop
# --------------------------------------------------------------------------------------


def solve(task, lang, provider, system, repairs, tokenizer, aval, timeout):
    messages = [{"role": "user", "content": first_prompt(task, lang)}]
    attempts = []
    total = len(task.hidden)
    for n in range(1, repairs + 2):
        prompt = messages[-1]["content"]
        rec = {"attempt": n, "prompt": prompt, "answer": None, "outcome": "failed", "reason": None,
               "compile_ok": False, "diagnostics": 0, "hidden_total": total, "failed_cases": [],
               "fault_kinds": [], "value_failures": 0,
               "tokens_in": 0, "tokens_out": 0, "tokens_system": 0, "tokens_source": tokenizer.name}
        try:
            reply = provider.complete(task, lang, n, system, messages)
        except ProviderError as e:
            rec["outcome"] = "provider_error"
            rec["reason"] = str(e)
            rec["failed_cases"] = list(range(total))
            attempts.append(rec)
            break
        rec["answer"] = reply.text
        rec["tokens_system"] = tokenizer.count(system)
        if reply.usage is not None:
            rec["tokens_in"], rec["tokens_out"] = reply.usage
            rec["tokens_source"] = "provider"
        else:
            rec["tokens_in"] = tokenizer.count(flatten(system, messages))
            rec["tokens_out"] = tokenizer.count(reply.text)
        code = extract_code(reply.text, lang)
        if code is None:
            rec["reason"] = "no_code"
            rec["failed_cases"] = list(range(total))
            failure = {"kind": "no_code"}
        else:
            res = (evaluate_aval(task, code, aval, timeout) if lang == "aval"
                   else evaluate_rust(task, code, timeout))
            rec["compile_ok"] = res["compile_ok"]
            rec["diagnostics"] = res["diagnostics"]
            rec["failed_cases"] = res["failed"]
            rec["fault_kinds"] = res.get("fault_kinds", [])
            rec["value_failures"] = res.get("value_failures", 0)
            if not res["failed"]:
                rec["outcome"] = "solved"
                attempts.append(rec)
                break
            rec["reason"] = res.get("reason", "hidden_failed")
            failure = res["failure"]
        attempts.append(rec)
        if n <= repairs:
            messages.append({"role": "assistant", "content": reply.text.strip() or "(empty reply)"})
            messages.append({"role": "user", "content": repair_prompt(lang, failure)})
    solved_at = next((a["attempt"] for a in attempts if a["outcome"] == "solved"), None)
    last = attempts[-1]
    resource = (solved_at is None and last["reason"] == "hidden_failed" and bool(last["fault_kinds"])
                and set(last["fault_kinds"]) <= set(RESOURCE_FAULTS) and last["value_failures"] == 0)
    perr = any(a["outcome"] == "provider_error" for a in attempts)
    if perr:
        solved_at = None
    return {
        "outcome": "provider_error" if perr else ("solved" if solved_at is not None else "failed"),
        "solved": solved_at is not None,
        "solved_at": solved_at,
        "compile_at_1": bool(attempts and attempts[0]["compile_ok"]),
        "pass_at_1": bool(attempts and attempts[0]["outcome"] == "solved"),
        "resource_failure": resource,
        "failed_cases": attempts[-1]["failed_cases"] if solved_at is None else [],
        "reason": None if solved_at is not None else attempts[-1]["reason"],
        "attempts": attempts,
    }


# --------------------------------------------------------------------------------------
# Metrics, results.json, RESULTS.md
# --------------------------------------------------------------------------------------


def rate(k, n):
    return k / n if n else 0.0


def metrics_for(results, lang):
    allrs = [r[lang] for r in results if lang in r]
    rs = [r for r in allrs if r["outcome"] != "provider_error"]
    n = len(rs)
    solved = sum(1 for r in rs if r["solved"])
    tin = sum(a["tokens_in"] for r in rs for a in r["attempts"])
    tout = sum(a["tokens_out"] for r in rs for a in r["attempts"])
    tsys = sum(a["tokens_system"] for r in rs for a in r["attempts"])
    sources = sorted({a["tokens_source"] for r in rs for a in r["attempts"]})
    reasons = {}
    for r in rs:
        if not r["solved"]:
            reasons[r["reason"] or "unknown"] = reasons.get(r["reason"] or "unknown", 0) + 1
    return {
        "tasks": n,
        "compile_at_1": rate(sum(1 for r in rs if r["compile_at_1"]), n),
        "pass_at_1": rate(sum(1 for r in rs if r["pass_at_1"]), n),
        "pass_at_r": rate(solved, n),
        "solved": solved,
        "pass_at_1_count": sum(1 for r in rs if r["pass_at_1"]),
        "tokens_in": tin,
        "tokens_out": tout,
        "tokens_per_solved": ((tin + tout) / solved) if solved else None,
        "tokens_per_solved_without_system": ((tin + tout - tsys) / solved) if solved else None,
        "provider_errors": len(allrs) - n,
        "resource_failures": sum(1 for r in rs if r.get("resource_failure")),
        "token_sources": sources,
        "unsolved_reasons": dict(sorted(reasons.items())),
    }


def verdict(results, langs):
    if "aval" not in langs or "rust" not in langs:
        return None
    both = [r for r in results if "aval" in r and "rust" in r]
    bad = sorted(r["id"] for r in results
                 if any(r.get(l, {}).get("outcome") == "provider_error" for l in langs))
    if bad:
        return {"criterion": "Aval pass@1 >= 0.8 x Rust pass@1", "result": "INCONCLUSIVE",
                "provider_error_tasks": bad}
    rs = both
    n = len(rs)
    a = sum(1 for r in rs if r["aval"]["pass_at_1"])
    b = sum(1 for r in rs if r["rust"]["pass_at_1"])
    if not n:
        return None
    if b == 0:
        return {"criterion": "Aval pass@1 >= 0.8 x Rust pass@1", "result": "INCONCLUSIVE",
                "reason": "Rust pass@1 is 0", "aval_pass_at_1": a / n, "rust_pass_at_1": 0.0,
                "provider_error_tasks": []}
    ok = Fraction(a, n) >= Fraction(4, 5) * Fraction(b, n)
    return {
        "criterion": "Aval pass@1 >= 0.8 x Rust pass@1",
        "aval_pass_at_1": a / n,
        "rust_pass_at_1": b / n,
        "threshold": 0.8 * (b / n),
        "result": "PASS" if ok else "FAIL",
        "degenerate": b == 0,
    }


BOOTSTRAP_RESAMPLES = 10000
BOOTSTRAP_SEED = 20260101


def task_set_name(tasks):
    """`HumanEval+` or `MBPP+`, inferred from the tasks' `source_tests` (else `mixed`/`unknown`)."""
    names = set()
    for t in tasks:
        st = t.data.get("source_tests", "")
        names.add("MBPP+" if "MBPP+" in st else "HumanEval+" if "HumanEval+" in st else "unknown")
    if len(names) == 1:
        return names.pop()
    return "mixed" if names else "unknown"


def bootstrap(results, langs):
    """95 % percentile bootstrap intervals over tasks (10,000 resamples, fixed seed) for each
    language's pass@1 and for the paired difference Aval - Rust. Pure standard library."""
    rows = [r for r in results if all(r.get(l, {}).get("outcome") != "provider_error" for l in langs)]
    n = len(rows)
    out = {"resamples": BOOTSTRAP_RESAMPLES, "seed": BOOTSTRAP_SEED, "tasks": n}
    if not n:
        return out
    ind = {l: [1 if r[l]["pass_at_1"] else 0 for r in rows] for l in langs}
    rng = random.Random(BOOTSTRAP_SEED)
    stats = {l: [] for l in langs}
    diffs = []
    for _ in range(BOOTSTRAP_RESAMPLES):
        idx = [int(rng.random() * n) for _ in range(n)]
        sums = {l: sum(ind[l][i] for i in idx) / n for l in langs}
        for l in langs:
            stats[l].append(sums[l])
        if "aval" in langs and "rust" in langs:
            diffs.append(sums["aval"] - sums["rust"])

    def interval(xs, point):
        xs = sorted(xs)
        lo = xs[int(0.025 * len(xs))]
        hi = xs[int(0.975 * len(xs)) - 1]
        return {"point": point, "low": lo, "high": hi}

    for l in langs:
        out[l] = interval(stats[l], sum(ind[l]) / n)
    if diffs:
        out["aval_minus_rust"] = interval(diffs, (sum(ind["aval"]) - sum(ind["rust"])) / n)
    return out


def fmt(x, digits=3):
    return "-" if x is None else ("%.*f" % (digits, x))


def render_markdown(doc):
    m = doc["metrics"]
    langs = doc["languages"]
    lines = ["# E2 results: Aval vs Rust, in-context", ""]
    lines.append("- Provider: `%s`%s" % (doc["provider"], " (model `%s`)" % doc["model"] if doc["model"] else ""))
    lines.append("Task set: %s" % doc["task_set"])
    lines.append("- Tasks: %d; repairs allowed: %d" % (doc["task_count"], doc["repairs"]))
    lines.append("- Skill (`aval skill print`) SHA-256: `%s`" % doc["skill_sha256"])
    lines.append("- Criterion (pre-registered): Aval pass@1 >= 0.8 x Rust pass@1")
    lines.append("")
    v = doc["verdict"]
    if v and v["result"] == "INCONCLUSIVE" and v.get("reason"):
        lines.append("Verdict: INCONCLUSIVE (%s; Aval pass@1 = %s, Rust pass@1 = %s)" % (
            v["reason"], fmt(v["aval_pass_at_1"]), fmt(v["rust_pass_at_1"])))
    elif v and v["result"] == "INCONCLUSIVE":
        lines.append("Verdict: INCONCLUSIVE (provider errors in %d task(s): %s; rerun them)" % (
            len(v["provider_error_tasks"]), ", ".join(v["provider_error_tasks"])))
    elif v:
        extra = " (degenerate: Rust pass@1 is 0)" if v["degenerate"] else ""
        lines.append("Verdict: %s (Aval pass@1 = %s, Rust pass@1 = %s, needs >= %s)%s" % (
            v["result"], fmt(v["aval_pass_at_1"]), fmt(v["rust_pass_at_1"]), fmt(v["threshold"]), extra))
    else:
        lines.append("Verdict: N/A (both languages must run on the same tasks)")
    lines.append("")
    head = "| Metric | " + " | ".join(l.capitalize() for l in langs) + " |"
    lines += [head, "| --- |" + " --- |" * len(langs)]
    rows = [
        ("compile@1", lambda x: fmt(x["compile_at_1"])),
        ("pass@1", lambda x: fmt(x["pass_at_1"])),
        ("pass@R (R = %d)" % doc["repairs"], lambda x: fmt(x["pass_at_r"])),
        ("solved tasks", lambda x: "%d / %d" % (x["solved"], x["tasks"])),
        ("tokens/solved (with system)", lambda x: fmt(x["tokens_per_solved"], 0)),
        ("tokens/solved (without system)", lambda x: fmt(x["tokens_per_solved_without_system"], 0)),
        ("resource failures (step/memory/stack limit only)", lambda x: str(x["resource_failures"])),
        ("provider errors (excluded)", lambda x: str(x["provider_errors"])),
        ("tokens in / out (total)", lambda x: "%d / %d" % (x["tokens_in"], x["tokens_out"])),
        ("token source", lambda x: ", ".join(x["token_sources"]) or "-"),
    ]
    for label, f in rows:
        lines.append("| %s | %s |" % (label, " | ".join(f(m[l]) for l in langs)))
    lines.append("")
    lines.append("tokens/solved (with system) = all tokens (prompt + answer, every attempt of every task) "
                 "divided by the number of solved tasks; the system prompt is the whole `aval skill print` "
                 "reference for Aval and one line for Rust. Without system: the system-prompt tokens of every "
                 "attempt are subtracted first.")
    lines.append("")
    for l in langs:
        if m[l]["unsolved_reasons"]:
            lines.append("Unsolved in %s by final reason: %s" % (
                l.capitalize(), ", ".join("%s=%d" % kv for kv in m[l]["unsolved_reasons"].items())))
    lines.append("")
    ci = doc["confidence"]
    if ci.get("tasks"):
        lines.append("Bootstrap over %d tasks (%d resamples, seed %d, percentile method):" % (
            ci["tasks"], ci["resamples"], ci["seed"]))
        lines.append("")
        for l in langs:
            if l in ci:
                lines.append("- %s pass@1 95%% CI: [%s, %s] (point estimate %s)" % (
                    l.capitalize(), fmt(ci[l]["low"], 4), fmt(ci[l]["high"], 4), fmt(ci[l]["point"], 4)))
        if "aval_minus_rust" in ci:
            d = ci["aval_minus_rust"]
            lines.append("- Aval \u2212 Rust 95%% CI: [%s, %s] (point estimate %s)" % (
                fmt(d["low"], 4), fmt(d["high"], 4), fmt(d["point"], 4)))
        lines.append("")
    lines += ["## Per task", "",
              "| Task | " + " | ".join("%s (solved at / attempts)" % l.capitalize() for l in langs) + " |",
              "| --- |" + " --- |" * len(langs)]
    for tid in sorted(doc["tasks"]):
        cells = []
        for l in langs:
            r = doc["tasks"][tid].get(l)
            if r is None:
                cells.append("-")
            elif r["outcome"] == "provider_error":
                cells.append("provider error (%s)" % r["reason"])
            elif r["solved"]:
                cells.append("%d / %d" % (r["solved_at"], len(r["attempts"])))
            else:
                cells.append("unsolved (%s) / %d" % (r["reason"], len(r["attempts"])))
        lines.append("| %s | %s |" % (tid, " | ".join(cells)))
    lines.append("")
    return "\n".join(lines)


def cmd_run(args):
    langs = [l.strip() for l in args.langs.split(",") if l.strip()]
    if not langs or any(l not in LANGS for l in langs):
        raise HarnessError("--langs must be a comma list of: aval, rust")
    only = []
    for o in args.task or []:
        only += [x for x in o.split(",") if x]
    tasks, errors = load_tasks(args.tasks, only or None)
    if errors and not only:
        for label in sorted(errors):
            for e in errors[label]:
                print("INVALID %s: %s" % (label, e), file=sys.stderr)
        raise HarnessError("task set is invalid; run validate-tasks")
    if not tasks:
        raise HarnessError("no tasks to run")
    aval = find_aval(args.aval_bin)
    rc, skill, err = run(aval + ["skill", "print"], os.getcwd(), args.timeout, None)
    if rc != 0:
        raise HarnessError("`aval skill print` failed: %s" % err.strip())
    skill_bytes = skill.encode("utf-8")
    skill_sha = hashlib.sha256(skill_bytes).hexdigest()

    if args.provider == "replay":
        if not args.responses:
            raise HarnessError("--provider replay needs --responses FILE|reference")
        provider = ReplayProvider(args.responses, tasks)
    elif args.provider == "command":
        pass_env = []
        for o in args.pass_env or []:
            pass_env += [x for x in o.split(",") if x]
        provider = CommandProvider(args.command, args.provider_timeout, pass_env)
    else:
        provider = AnthropicProvider(args.model, args.max_tokens, args.provider_timeout)
    tokenizer = Tokenizer(args.tokenizer)

    generated = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    def write_outputs(results):
        doc = {
            "schema": 1,
            "generated_at": generated,
            "provider": provider.name,
            "model": args.model if args.provider == "anthropic" else None,
            "skill_sha256": skill_sha,
            "repairs": args.repairs,
            "languages": langs,
            "task_set": task_set_name(tasks),
            "task_count": len(tasks),
            "tasks_done": len(results),
            "metrics": {l: metrics_for(results, l) for l in langs},
            "verdict": verdict(results, langs),
            "confidence": bootstrap(results, langs),
            "tasks": {r["id"]: {l: r[l] for l in langs} for r in results},
        }
        os.makedirs(args.out, exist_ok=True)
        # write-then-rename: an interrupted run never leaves a truncated file
        for name, text in (("results.json",
                            json.dumps(doc, indent=2, sort_keys=True, ensure_ascii=False) + "\n"),
                           ("RESULTS.md", render_markdown(doc))):
            tmp = os.path.join(args.out, name + ".tmp")
            with open(tmp, "w", encoding="utf-8") as f:
                f.write(text)
            os.replace(tmp, os.path.join(args.out, name))
        return doc

    results = []
    for t in tasks:
        row = {"id": t.id}
        for lang in langs:
            system = skill if lang == "aval" else RUST_SYSTEM
            row[lang] = solve(t, lang, provider, system, args.repairs, tokenizer, aval, args.timeout)
            r = row[lang]
            print("%s %-4s %s" % (t.id, lang, "solved@%d" % r["solved_at"] if r["solved"]
                                  else "%s (%s)" % ("provider error" if r["outcome"] == "provider_error"
                                                    else "unsolved", r["reason"])),
                  file=sys.stderr)
        results.append(row)
        doc = write_outputs(results)
    for l in langs:
        m = doc["metrics"][l]
        print("%s: compile@1=%s pass@1=%s pass@R=%s tokens/solved=%s" % (
            l, fmt(m["compile_at_1"]), fmt(m["pass_at_1"]), fmt(m["pass_at_r"]),
            fmt(m["tokens_per_solved"], 0)))
    if doc["verdict"]:
        v = doc["verdict"]
        if v["result"] == "INCONCLUSIVE":
            print("Verdict: INCONCLUSIVE (%s)" % (v.get("reason") or "provider errors"))
        else:
            print("Verdict: %s (Aval pass@1 = %s, Rust pass@1 = %s)" % (
                v["result"], fmt(v["aval_pass_at_1"]), fmt(v["rust_pass_at_1"])))
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description="E2 in-context evaluation harness")
    sub = ap.add_subparsers(dest="cmd")
    v = sub.add_parser("validate-tasks", help="check every task file against the schema")
    v.add_argument("--tasks", default=DEFAULT_TASKS, help="task directory (default: tasks/ next to this file)")
    r = sub.add_parser("run", help="run tasks against a provider")
    r.add_argument("--tasks", default=DEFAULT_TASKS)
    r.add_argument("--task", "--only", action="append", metavar="ID",
                   help="run only these task ids (repeatable or comma separated)")
    r.add_argument("--langs", "--lang", default="aval,rust", help="comma list: aval,rust (default both)")
    r.add_argument("--provider", choices=["replay", "command", "anthropic"], required=True)
    r.add_argument("--responses", help="replay: JSON file, or the word `reference`")
    r.add_argument("--command", help="command provider: shell command reading the prompt on stdin")
    r.add_argument("--pass-env", action="append", metavar="NAME",
                   help="command provider: also pass this environment variable (repeatable or comma separated)")
    r.add_argument("--model", help="anthropic: model id")
    r.add_argument("--max-tokens", type=int, default=4096)
    r.add_argument("--repairs", type=int, default=3, help="repair rounds after the first attempt (default 3)")
    r.add_argument("--out", default=HERE, help="directory for results.json and RESULTS.md (default: this directory)")
    r.add_argument("--aval-bin", help="aval command (default: $AVAL_BIN, target/debug/aval, PATH, cargo run)")
    r.add_argument("--tokenizer", choices=["auto", "tiktoken", "estimate"], default="auto")
    r.add_argument("--timeout", type=int, default=60, help="seconds per aval/rustc/test run (default 60)")
    r.add_argument("--provider-timeout", type=int, default=300, help="seconds per provider call")
    args = ap.parse_args(argv)
    if args.cmd is None:
        ap.print_help()
        return 2
    if args.cmd == "run" and args.repairs < 0:
        print("error: --repairs must be >= 0", file=sys.stderr)
        return 2
    try:
        return cmd_validate(args) if args.cmd == "validate-tasks" else cmd_run(args)
    except HarnessError as e:
        print("error: %s" % e, file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
