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

## Output format and edge cases (architect decisions)

- `provider` is part of the same-configuration check, with task ids, skill hash, repairs and model.
- A rejected invocation exits non-zero, names the differing field in its message, and writes nothing
  to `--out`.
- `aggregate.json`: `{"runs": N, "languages": {"aval": {"pass_at_1": [...], "mean": x, "stdev": x|null,
  "flip_rate": x}, "rust": {...}}, "verdict": {"mean": "PASS"|"FAIL", "every_run": true|false}}`.
- `AGGREGATE.md`: one table row per run, per-language summary lines, then the lines
  `Verdict on mean: PASS|FAIL` and `Every run satisfies the criterion: yes|no`.
- The flip rate is per language: tasks whose first-attempt outcome differs between runs, divided by the
  task count. Tasks with a provider error in any run are excluded from both numerator and denominator.
- Fewer than 3 runs: the report is produced, `stdev` is null (`n/a` in the table) and a warning says the
  deviation is not meaningful.
- A run whose own verdict is INCONCLUSIVE is rejected (it carries no usable pass rate).
- If the mean Rust pass@1 is 0, the criterion holds trivially and the report says so.
- Per-run pass@1 is read from each file's `metrics.<lang>.pass_at_1` and is not recomputed; a task with a
  provider error counts there as the original run counted it.
- The mean is the arithmetic mean of the per-run values (not pooled over tasks).
- When the mean Rust pass@1 is 0 the report contains the line `Rust pass@1 is 0: criterion holds trivially`.

## Review follow-ups (architect decisions)

- The criterion `aval >= 0.8 x rust` is evaluated with exact rational arithmetic, as in `harness.py`
  (`Fraction`), for the mean and for each run. Floats must not decide a boundary case.
- Duplicate inputs (same path, or identical file content) are rejected: repeated runs must be distinct.
- Malformed input (invalid JSON, missing `metrics` or language arms, non-numeric pass rates, `tasks` of
  the wrong type) is rejected with a one-line message and exit status 1, never a traceback; nothing is
  written.
- The report and stderr state when the model identity could not be verified because every run records
  `model: null` (command provider): `Model identity not verified`.
- A flip rate with zero countable tasks is reported as `n/a` (null in JSON).
- The INCONCLUSIVE rejection stays. The provider-error exclusion and the Rust-mean-0 line therefore
  only matter for hand-made files; they are kept for robustness and exercised by fixtures.

## Second review follow-ups (architect decisions)

- Task-level `pass_at_1` is validated like the metric: every arm of every task must carry a boolean
  `pass_at_1`; a missing, string, null or NaN value rejects the file (EVH-046).
- Output is all-or-nothing: both files are rendered first, written to temporary files in `--out` and
  moved into place; any `OSError` (for example `--out` is a file, or a target is a directory) gives exit
  status 1, a one-line message and no traceback, and leaves no new or modified file (EVH-048).
