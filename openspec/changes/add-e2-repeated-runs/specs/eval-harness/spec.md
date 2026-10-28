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
