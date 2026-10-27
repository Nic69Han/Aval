# cli Specification

## Purpose
Defines the `aval` command-line interface used by agents and humans, and the compact language
reference embedded in the binary.

## Requirements

### Requirement: Commands
The `aval` binary SHALL provide `check <file>`, `test <file> [--cases N] [--seed S]`, `spec lock <file>`, `skill print`, and `--version`.

#### Scenario: CLI-001 Unknown command
- **WHEN** `aval frobnicate` is run
- **THEN** the exit code is 2 and a usage message is printed on stderr

#### Scenario: CLI-002 Version
- **WHEN** `aval --version` is run
- **THEN** stdout starts with `aval 0.1.`

### Requirement: Embedded skill reference
`aval skill print` SHALL print the compact reference compiled into the binary, containing the grammar, the keyword list and the diagnostic code table, within 4,000 o200k tokens.

#### Scenario: CLI-003 Skill contains every diagnostic code
- **WHEN** `aval skill print` is run
- **THEN** stdout contains every diagnostic code implemented by the MVP and all 46 keywords

#### Scenario: CLI-004 Skill size budget
- **WHEN** the output of `aval skill print` is tokenized with o200k_base
- **THEN** it has at most 4,000 tokens

#### Scenario: CLI-005 Skill code blocks are valid Aval
- **WHEN** each fenced `aval` code block of the skill output is checked with `aval check`
- **THEN** none reports an error

### Requirement: Test case count option
`--cases N` SHALL set the number of generated inputs per function, and `--seed S` the RNG seed.

#### Scenario: CLI-006 Zero generated cases still runs examples
- **WHEN** `aval test --cases 0` runs on a file with a failing example
- **THEN** `aval test` reports `T100`

### Requirement: Configurable step budget
`aval test` SHALL accept `--max-steps N` to set the per-input step budget (default 10,000,000).

#### Scenario: CLI-007 Raising the budget lets a slow function finish
- **WHEN** a function needing about 30 million steps on its example is run with `aval test --max-steps 50000000`
- **THEN** `aval test` reports no `T700`, while the default budget reports `T700` with `fault` = `step_limit`
