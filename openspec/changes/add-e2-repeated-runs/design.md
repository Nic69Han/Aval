## Decisions

- **Aggregate outside the harness.** Repeats are plain re-runs with different `--out` directories;
  the aggregator only reads `results.json`. The harness and its attestation stay untouched.
- **Same-configuration check.** The aggregator compares task ids, skill SHA-256, repairs, provider
  and model fields recorded in each `results.json`, and exits non-zero on any mismatch.
- **Statistic.** Mean and sample standard deviation of per-run pass@1 per language; flip rate =
  tasks solved at attempt 1 in some but not all runs, divided by task count. Fewer than 3 runs
  prints a warning that the deviation is not meaningful.
- **Verdict.** The pre-registered criterion is evaluated on the mean pass@1; the aggregator also
  states whether it holds for every individual run.
