## ADDED Requirements

### Requirement: Aggregation of repeated runs
The experiment tooling SHALL provide an aggregator that combines the results of several runs of one
configuration and reports the variation between them.

#### Scenario: EVH-040 Mismatched runs are rejected
- **WHEN** the aggregator is given results files whose task ids, skill hash, repair count or model differ
- **THEN** it exits with a non-zero status and names the differing field

#### Scenario: EVH-041 Per-run and summary statistics
- **WHEN** the aggregator is given three or more matching results files
- **THEN** its report lists pass@1 of each run per language, the mean, the sample standard deviation and the share of tasks whose first-attempt outcome differs between runs

#### Scenario: EVH-042 Verdict on the mean
- **WHEN** the aggregator evaluates the pre-registered criterion
- **THEN** it applies it to the mean pass@1 and states separately whether every individual run satisfies it

#### Scenario: EVH-043 Fewer than three runs
- **WHEN** the aggregator is given one or two matching results files
- **THEN** it still writes its report, shows the standard deviation as not available and prints a warning that the deviation is not meaningful

#### Scenario: EVH-044 Inconclusive runs are rejected
- **WHEN** one of the results files carries the verdict INCONCLUSIVE
- **THEN** the aggregator exits with a non-zero status, names the file and writes nothing

#### Scenario: EVH-045 Exact criterion at the boundary
- **WHEN** a mean or per-run pass rate equals exactly 0.8 times the Rust rate, for example 12 of 20 against 15 of 20
- **THEN** the criterion holds in both verdicts

#### Scenario: EVH-046 Invalid input is rejected cleanly
- **WHEN** the aggregator is given a duplicate file, invalid JSON, or a file missing a required field
- **THEN** it exits with status 1, prints a one-line message naming the file, prints no traceback and writes nothing

#### Scenario: EVH-047 Unverified model identity is stated
- **WHEN** every results file records no model
- **THEN** the report and the standard error state that the model identity is not verified

#### Scenario: EVH-048 Output failures are clean and atomic
- **WHEN** the output directory cannot be used, for example because it is an existing file or one of the target files is a directory
- **THEN** the aggregator exits with status 1, prints a one-line message and no traceback, and leaves any existing output files unchanged
