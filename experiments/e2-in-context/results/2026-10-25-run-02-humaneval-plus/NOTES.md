# Run 02 — HumanEval+ hidden cases

Same provider and model as run 01 (`providers/claude-cli.sh`), same 40 tasks and statements; the
hidden cases are now the HumanEval+ cases expressible in Aval (29,363 instead of 328), compared with
EvalPlus tolerance, run in batches with a per-case step budget. Release `aval` binary.

| Metric | Run 01 (Aval / Rust) | Run 02 (Aval / Rust) |
| --- | --- | --- |
| hidden cases | 328 / 328 | 29,363 / 29,363 |
| compile@1 | 0.950 / 1.000 | 1.000 / 1.000 |
| pass@1 | 0.950 / 1.000 | **0.975 / 1.000** |
| pass@3 | 1.000 / 1.000 | 1.000 / 1.000 |
| resource failures | — | 0 / 0 |
| tokens/solved, with system | 2,931 / 220 | 2,828 / 223 |
| tokens/solved, without system | 343 / 193 | 310 / 196 |

**Verdict: PASS** (criterion Aval pass@1 ≥ 0.8 × Rust pass@1).

The only Aval first-attempt failure: HE-046 (`fib4`), 2 of 64 cases with `overflow` — the answer
computed one term too many, the same defect the original reference had and that the 8 HumanEval cases
of run 01 could not reveal. The repair prompt ("2 of 64 cases failed") was enough to fix it. Rust's
answer used a different formulation and did not overflow.

The first attempt at run 02 (not kept) returned `INCONCLUSIVE` because the provider failed on 35 tasks
(account usage limit); the harness excluded them as designed and the run was repeated in full.

## Caveats

Run-to-run variation is not measured (one run per configuration, default sampling of the CLI), so
the difference between runs 01 and 02 is not significant. HumanEval and HumanEval+ are likely in the
model's training data. Large-n stress cases are excluded for HE-049 and HE-036 (both arms). See the
harness README, "Threats to validity".
