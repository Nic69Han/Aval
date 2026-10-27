#!/usr/bin/env python3
"""Experiment E1: is each Aval reserved word a single token?

Measures every keyword in keywords.txt (plus the project name) in three contexts:
  bare            "fn"
  leading_space   " fn"
  after_newline   "\\nfn"   (raw count, and delta = raw - count("\\n"))

No special tokens are added. Writes results.json and RESULTS.md next to this file.

Usage (from a venv with requirements.txt installed):
    python -I measure.py
Network is needed on first run to download tokenizer files.
Claude is measured only when ANTHROPIC_API_KEY is set (count_tokens endpoint).
"""
import datetime
import json
import os
import sys
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
KEYWORDS_FILE = HERE / "keywords.txt"
RESULTS_JSON = HERE / "results.json"
RESULTS_MD = HERE / "RESULTS.md"

TIKTOKEN_ENCODINGS = ["o200k_base", "cl100k_base"]
HF_REPOS = [
    "unsloth/Llama-3.2-1B",          # Llama 3.2 tokenizer, non-gated mirror
    "NousResearch/Meta-Llama-3-8B",  # Llama 3 tokenizer, non-gated mirror
    "Qwen/Qwen2.5-7B-Instruct",
    "Qwen/Qwen3-8B",
    "deepseek-ai/DeepSeek-V3",
    "mistralai/Mistral-7B-v0.3",
]
CLAUDE_MODEL = "claude-haiku-5-5"
MAX_TWO_TOKEN = 3  # success criterion: at most 3 keywords at 2 tokens per tokenizer


def load_terms():
    keywords, project = [], None
    for raw in KEYWORDS_FILE.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.lower().startswith("project:"):
            project = line.split(":", 1)[1].strip()
            continue
        keywords.append(line)
    return keywords, project


def load_tiktoken(enc_name):
    import tiktoken

    enc = tiktoken.get_encoding(enc_name)

    def count(s):
        return len(enc.encode(s, disallowed_special=()))

    def pieces(s):
        return [enc.decode([i]) for i in enc.encode(s, disallowed_special=())]

    info = {"library": f"tiktoken {tiktoken.__version__}", "encoding": enc_name}
    return count, pieces, info


def load_hf(repo):
    from huggingface_hub import HfApi

    try:
        revision = HfApi().model_info(repo).sha
    except Exception as e:  # revision is informational only
        revision = f"unknown ({type(e).__name__})"

    errors = []
    try:
        from transformers import AutoTokenizer

        tok = AutoTokenizer.from_pretrained(repo)

        def count(s):
            return len(tok.encode(s, add_special_tokens=False))

        def pieces(s):
            return tok.convert_ids_to_tokens(tok.encode(s, add_special_tokens=False))

        method = "transformers.AutoTokenizer"
    except Exception as e:
        errors.append(f"AutoTokenizer: {type(e).__name__}: {str(e)[:300]}")
        try:
            from tokenizers import Tokenizer

            tok = Tokenizer.from_pretrained(repo)

            def count(s):
                return len(tok.encode(s, add_special_tokens=False).ids)

            def pieces(s):
                return tok.encode(s, add_special_tokens=False).tokens

            method = "tokenizers.Tokenizer.from_pretrained (fallback)"
        except Exception as e2:
            raise RuntimeError(" | ".join(errors + [f"tokenizers fallback: {type(e2).__name__}: {str(e2)[:300]}"]))

    import transformers
    import tokenizers

    info = {
        "library": f"transformers {transformers.__version__} / tokenizers {tokenizers.__version__}",
        "repo": repo,
        "revision": revision,
        "method": method,
        "fallback_errors": errors,
    }
    return count, pieces, info


def load_claude():
    key = os.environ.get("ANTHROPIC_API_KEY")
    if not key:
        return None  # recorded as "not measured (no API key)"

    def raw(text):
        body = json.dumps({"model": CLAUDE_MODEL, "messages": [{"role": "user", "content": text}]}).encode("utf-8")
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages/count_tokens",
            data=body,
            headers={"x-api-key": key, "anthropic-version": "2023-06-01", "content-type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=60) as resp:
            return json.load(resp)["input_tokens"]

    # Subtract a baseline message; "." is assumed to be one token (approximate).
    base = raw(".")

    def count(s):
        return raw(s) - base + 1

    info = {"library": "Anthropic Messages count_tokens", "model": CLAUDE_MODEL,
            "method": "approximate: count(text) - count('.') + 1"}
    return count, None, info


def measure(count, pieces, terms):
    nl = count("\n")
    rows = {}
    for t in terms:
        rec = {
            "bare": count(t),
            "leading_space": count(" " + t),
            "after_newline_raw": count("\n" + t),
        }
        rec["after_newline_delta"] = rec["after_newline_raw"] - nl
        if pieces:
            rec["pieces"] = {"bare": pieces(t), "leading_space": pieces(" " + t)}
        rows[t] = rec
    return rows, nl


def summarize(rows):
    vals = list(rows.values())
    two_bare = [t for t, r in rows.items() if r["bare"] == 2]
    two_lead = [t for t, r in rows.items() if r["leading_space"] == 2]
    gt2_bare = [t for t, r in rows.items() if r["bare"] > 2]
    gt2_lead = [t for t, r in rows.items() if r["leading_space"] > 2]
    gt1_any = [t for t, r in rows.items() if r["bare"] > 1 or r["leading_space"] > 1]
    return {
        "two_token_bare": two_bare,
        "two_token_leading_space": two_lead,
        "over_two_tokens_bare": gt2_bare,
        "over_two_tokens_leading_space": gt2_lead,
        "over_one_token_any_context": gt1_any,
        "single_token_leading_space": sum(1 for r in vals if r["leading_space"] == 1),
        # Primary verdict: code context (keyword preceded by whitespace).
        "verdict": "PASS" if len(two_lead) <= MAX_TWO_TOKEN else "FAIL",
        # Secondary verdict: keyword at the start of the text, no leading space.
        "verdict_bare": "PASS" if len(two_bare) <= MAX_TWO_TOKEN else "FAIL",
    }


def main():
    import platform

    keywords, project = load_terms()
    terms = keywords + ([project] if project else [])
    now = datetime.datetime.now(datetime.timezone.utc)

    adapters = [(f"tiktoken:{e}", (lambda e=e: load_tiktoken(e))) for e in TIKTOKEN_ENCODINGS]
    adapters += [(f"hf:{r}", (lambda r=r: load_hf(r))) for r in HF_REPOS]
    adapters += [("claude:count_tokens", load_claude)]

    out = {
        "generated_utc": now.isoformat(timespec="seconds"),
        "python": sys.version.split()[0],
        "platform": platform.platform(),
        "keywords": keywords,
        "project_name": project,
        "contexts": ["bare", "leading_space", "after_newline_raw", "after_newline_delta"],
        "max_two_token_criterion": MAX_TWO_TOKEN,
        "tokenizers": {},
    }

    for name, loader in adapters:
        print(f"[E1] {name} ...", flush=True)
        try:
            loaded = loader()
            if loaded is None:
                out["tokenizers"][name] = {"status": "not measured (no API key)"}
                print("  not measured (no API key)", flush=True)
                continue
            count, pieces, info = loaded
            rows, nl = measure(count, pieces, terms)
            entry = {"status": "ok", **info, "newline_token_count": nl, "rows": rows, "summary": summarize(rows)}
            out["tokenizers"][name] = entry
            print(f"  ok: {len(rows)} terms, verdict {entry['summary']['verdict']}", flush=True)
        except Exception as e:
            out["tokenizers"][name] = {"status": "failed", "error": f"{type(e).__name__}: {str(e)[:500]}"}
            print(f"  FAILED: {type(e).__name__}: {str(e)[:200]}", flush=True)

    RESULTS_JSON.write_text(json.dumps(out, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    RESULTS_MD.write_text(render_markdown(out), encoding="utf-8")
    print(f"wrote {RESULTS_JSON} and {RESULTS_MD}", flush=True)


def short(name):
    return name.split(":", 1)[1].split("/")[-1] if ":" in name else name


def e_nl(entry):
    return entry["newline_token_count"]


def render_markdown(out):
    tk = out["tokenizers"]
    names = list(tk)
    ok = [n for n in names if tk[n]["status"] == "ok"]
    failed = [n for n in names if tk[n]["status"] == "failed"]
    skipped = [n for n in names if tk[n]["status"].startswith("not measured")]
    terms = out["keywords"] + ([out["project_name"]] if out["project_name"] else [])

    L = []
    L.append("# E1 results: are Aval reserved words single tokens?")
    L.append("")
    L.append(f"- Date (UTC): {out['generated_utc'][:10]} (run at {out['generated_utc']})")
    L.append(f"- Inputs: `keywords.txt` ({len(out['keywords'])} keywords) and project name `{out['project_name']}`")
    L.append(f"- Script: `measure.py`, Python {out['python']}, {out['platform']}")
    L.append("- Contexts: bare (`fn`), leading space (` fn`), after newline (`\\nfn`). The after-newline delta is the raw count minus the count of `\\n` alone; it is exact only if the newline does not merge with the keyword.")
    L.append("- No special tokens (no BOS) are added. Cell format in the main table: `bare / leading-space` token counts.")
    L.append("")
    L.append("## Verdict")
    L.append("")
    L.append(f"Success criterion: at most {MAX_TWO_TOKEN} keywords at 2 tokens per tokenizer. "
             "The primary verdict uses the leading-space context, because in code a keyword is usually preceded by whitespace. "
             "The bare-context verdict is reported as a secondary check. "
             "Keywords at 3 or more tokens are outside the literal criterion and are shown in the last two count columns.")
    L.append("")
    L.append("| Tokenizer | Status | Single-token (leading space) | 2 tokens (leading space) | Verdict (leading space) | 2 tokens (bare) | Verdict (bare, secondary) | >2 tokens (bare / leading) |")
    L.append("|---|---|---|---|---|---|---|---|")
    for n in names:
        e = tk[n]
        if e["status"] != "ok":
            L.append(f"| `{n}` | {e['status']} | - | - | not measured | - | - | - |")
            continue
        s = e["summary"]
        L.append(f"| `{n}` | ok | {s['single_token_leading_space']}/{len(terms)} | {len(s['two_token_leading_space'])} | "
                 f"**{s['verdict']}** | {len(s['two_token_bare'])} | {s['verdict_bare']} | "
                 f"{len(s['over_two_tokens_bare'])} / {len(s['over_two_tokens_leading_space'])} |")
    L.append("")
    if ok:
        passing = [n for n in ok if tk[n]["summary"]["verdict"] == "PASS"]
        L.append(f"Overall (leading space): {len(passing)} of {len(ok)} measured tokenizers PASS. "
                 f"{len(failed)} failed to load and {len(skipped)} not measured, so coverage is partial.")
    L.append("")

    L.append("## Keyword x tokenizer table (bare / leading space)")
    L.append("")
    head = ["Term"] + [short(n) for n in names]
    L.append("| " + " | ".join(head) + " |")
    L.append("|" + "---|" * len(head))
    for t in terms:
        cells = [f"`{t}`"]
        for n in names:
            e = tk[n]
            if e["status"] != "ok":
                cells.append("n/a")
            else:
                r = e["rows"][t]
                cells.append(f"{r['bare']} / {r['leading_space']}")
        L.append("| " + " | ".join(cells) + " |")
    L.append("")
    L.append("Column headers: " + "; ".join(f"`{short(n)}` = `{n}`" for n in names) + ".")
    L.append("")

    L.append("## Terms with more than one token, per tokenizer")
    L.append("")
    L.append("Listed when the bare or leading-space count is above 1. Pieces are shown for the leading-space form.")
    L.append("")
    for n in ok:
        e = tk[n]
        items = e["summary"]["over_one_token_any_context"]
        L.append(f"**`{n}`** ({len(items)} terms)")
        if not items:
            L.append("- none")
        for t in items:
            r = e["rows"][t]
            pcs = " ".join(p.replace("|", "\\|") for p in r.get("pieces", {}).get("leading_space", []))
            L.append(f"- `{t}`: bare {r['bare']}, leading space {r['leading_space']} (pieces: `{pcs}`)")
        L.append("")
    for n in failed + skipped:
        L.append(f"**`{n}`**: {tk[n]['status']}")
        L.append("")

    L.append("## After-newline context")
    L.append("")
    L.append("Number of terms whose after-newline delta (`\\n` + term minus `\\n`) is exactly 1 token:")
    L.append("")
    for n in ok:
        e = tk[n]
        single = sum(1 for t in terms if e["rows"][t]["after_newline_delta"] == 1)
        L.append(f"- `{n}`: {single}/{len(terms)} (the `\\n` token alone counts {e['newline_token_count']})")
    odd = [n for n in ok if e_nl(tk[n]) != 1]
    if odd:
        L.append("Note: where the `\\n` token alone is not 1 token (" + ", ".join(f"`{n}`" for n in odd) +
                 "), the delta subtracts that count, so this column is less reliable for those tokenizers.")
    L.append("")

    L.append("## Failures and limits")
    L.append("")
    if failed:
        for n in failed:
            L.append(f"- `{n}` FAILED: {tk[n]['error']}")
    else:
        L.append("- No tokenizer failed to load.")
    for n in skipped:
        L.append(f"- `{n}`: {tk[n]['status']}")
    L.append("- Claude row: " + ("measured via count_tokens (approximate, see entry in results.json)" if "claude:count_tokens" in ok
                                 else "not measured, because ANTHROPIC_API_KEY was not set in this run."))
    L.append("")

    L.append("## Versions and repository ids")
    L.append("")
    L.append("| Tokenizer | Library | Repo / encoding | Revision (commit) | Method |")
    L.append("|---|---|---|---|---|")
    for n in names:
        e = tk[n]
        if e["status"] != "ok":
            L.append(f"| `{n}` | - | - | - | {e['status']} |")
            continue
        repo = e.get("repo") or e.get("encoding") or e.get("model", "")
        L.append(f"| `{n}` | {e['library']} | `{repo}` | `{e.get('revision', '-')}` | {e.get('method', 'tiktoken get_encoding')} |")
    L.append("")
    L.append("Reproduce: `python -I measure.py` inside a venv with `requirements.txt` installed. Raw per-keyword data, including pieces and after-newline values, is in `results.json`.")
    L.append("")
    return "\n".join(L)


if __name__ == "__main__":
    main()
