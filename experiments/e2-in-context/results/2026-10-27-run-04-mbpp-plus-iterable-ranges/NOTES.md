# Run 04 — MBPP+ after `add-iterable-ranges-and-not-operand`

Same 236 MBPP+ tasks, provider and model as run 03; the compiler has iterable ranges and
`a == not b`, and `aval skill print` describes them (2,529 o200k tokens, was ~2,460). Release binary.

| Metric | Run 03 (Aval / Rust) | Run 04 (Aval / Rust) |
| --- | --- | --- |
| compile@1 | 0.987 / 1.000 | **0.996** / 1.000 |
| pass@1 | 0.911 / 0.932 | 0.907 / 0.949 |
| pass@3 | 0.987 / 0.992 | 0.983 / 0.996 |
| Aval − Rust pass@1, 95 % CI | −0.021 [−0.059, +0.017] | −0.042 [−0.076, −0.009] |
| first-attempt failures | 21 / 16 | 22 / 12 |

**Verdict: PASS** (criterion Aval pass@1 ≥ 0.8 × Rust pass@1 = 0.759).

## What the change did

- The failures it targets are gone: MB-084 and MB-260 (`E204` on `(a..b).fold`) and MB-804
  (`== not`) now pass at the first attempt. Models used range methods in 63 first answers.
- First-attempt compile errors fell from 3 to 1 (compile@1 0.987 → 0.996). The remaining one (MB-224)
  is `n > 0 == (result > 0)`, a chained comparison that Rust rejects too.
- pass@1 did **not** improve: five other tasks failed at the first attempt this time (MB-224, 244,
  556, 563, 755; wrong values, `overflow` on `f64` inputs, one chained comparison), none related to
  ranges or `not`.

## Reading the two runs together

The 95 % intervals only cover task sampling, not the model's sampling noise. Between runs 03 and 04
Rust changed on 6 tasks (−5, +1) and Aval on 9 (−4, +5), so run-to-run noise is about ±5 tasks
(±2 points) — the same size as the effect of the change. Run 03 gave an interval containing 0, run 04
an interval excluding it; averaging the two runs gives a gap of about −3 points (Aval 0.909, Rust
0.940). The honest summary is: **on MBPP+ Aval is a few points behind Rust at the first attempt, and
equal after repairs (pass@3 0.983 vs 0.996)**. Separating a 2-point effect from noise needs several
runs per configuration.

## Remaining Aval-only causes (13 tasks at first attempt)

| Cause | Tasks |
| --- | --- |
| 64-bit `Int` overflow on one large case | MB-067, MB-608, MB-752, MB-244 |
| `Float` input overflowing `Int` conversion | MB-755 |
| wrong values (statement reading, tie rules) | MB-269, 556, 563, 604, 623, 749, 786 |
| chained comparison | MB-224 |

Language candidates, by frequency: wider integers (`i128`), a clearer statement of rounding and tie
rules in task prompts (task-side, not language-side).
