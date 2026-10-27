# Run 03 — MBPP+ (236 tasks)

Same provider and model as runs 01–02 (`providers/claude-cli.sh`), release `aval` binary, up to 3
repairs. Task set: 236 MBPP+ tasks (EvalPlus, Apache-2.0; MBPP, CC-BY-4.0), ~100 hidden cases each.

| Metric | Aval | Rust |
| --- | --- | --- |
| compile@1 | 0.987 | 1.000 |
| pass@1 | **0.911** [0.873, 0.945] | **0.932** [0.898, 0.962] |
| pass@3 | 0.987 | 0.992 |
| resource failures | 0 | 0 |
| tokens/solved, with / without system | 3,305 / 426 | 300 / 270 |

Aval − Rust pass@1: −0.021, 95 % CI [−0.059, +0.017] (paired bootstrap over tasks). The interval
contains 0: no significant difference between the languages on this set. **Verdict: PASS**
(criterion Aval pass@1 ≥ 0.8 × Rust pass@1 = 0.746).

Unlike HumanEval+, MBPP+ is below the ceiling for both languages, so the comparison is informative.

## First-attempt failures

- Both languages: 9 tasks (MB-074, 103, 559, 589, 593, 611, 748, 771, 800) — task difficulty or
  ambiguity, not language. MB-074 and MB-771 stay unsolved after repairs in both languages.
- Aval only: 12 tasks; Rust only: 7 tasks.

Aval-only failures by cause:

| Cause | Tasks | Language finding |
| --- | --- | --- |
| `E204` on `(a..b).fold` — ranges have no methods | MB-084, MB-260 | Same failure as run 01: make ranges iterable (`(0..n).fold/map/sum`) |
| `overflow` on 1 large case | MB-067, MB-608, MB-752 | Rust answers can use `i128`/`u128`; Aval has only 64-bit `Int` |
| `E002` on `== not x` | MB-804 | `not` is only allowed at the start of a conjunction operand; models write `a == not b` |
| wrong values | MB-269, 564, 604, 623, 749, 786 | to inspect individually (likely task-reading differences) |

All 12 were repaired from the diagnostics (pass@3 0.987).

## Caveats

One run, default CLI sampling: the intervals cover task sampling only, not sampling noise of the
model. MBPP is likely in the model's training data. Statements are MBPP's terse prompts plus one
example; 14 + 7 tasks were excluded for statement/hidden contradictions (see `tasks-mbpp/EXCLUDED.md`).
