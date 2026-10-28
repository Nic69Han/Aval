## Why

Each E2 configuration has been run once. The reported intervals resample tasks only; they do not
capture the variation of the model's own sampling. A difference such as 0.907 vs 0.949 cannot be
told apart from run-to-run noise.

## What Changes

- Add `tools/aggregate_runs.py`: combines the `results.json` files of N runs of one configuration and
  reports per-run pass@1, mean, sample standard deviation, and the share of tasks whose outcome
  differs between runs.
- Refuse to combine runs that differ in task set, skill hash, repair count or model.
- No change to `harness.py` or to the pre-registered criterion.

## Impact

- Affected specs: eval-harness.
- Affected code: `experiments/e2-in-context/tools/`.
- Cost: N extra provider runs per configuration; no change to existing results.
