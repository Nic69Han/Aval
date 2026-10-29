## 1. Aggregator
- [x] 1.1 Write `tools/aggregate_runs.py` (reads N `results.json`, writes `AGGREGATE.md` and `aggregate.json`)
- [x] 1.2 Reject mismatched configurations (EVH-040)

## 2. Tests
- [x] 2.1 Tests for scenarios EVH-040 to EVH-048 using small fixture results files

## 3. Runs
- [ ] 3.1 Re-run MBPP+ (current skill) twice more into dated directories and aggregate
- [ ] 3.2 Write the outcome into a NOTES.md next to the aggregate

## 4. Follow-ups from review (non-blocking)
- [ ] 4.1 EVH-040: task-id and task-count cases also assert no traceback and a one-line message
- [ ] 4.2 Remove the empty `--out` directories created by the tool when the second replace fails
- [ ] 4.3 Tests that inject a late write failure (restore and temp-file cleanup paths)
