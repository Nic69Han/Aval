# diagnostics Specification

## Purpose
Defines diagnostics as a stable machine API: one compact JSON object per line on standard output,
with a stable code, a position and code-specific fields, plus deterministic exit codes.

## Requirements

### Requirement: One JSON object per line
`aval check` and `aval test` SHALL print each diagnostic as a single-line JSON object on stdout, with at least `code`, `at` (`[line, column]`, 1-based) and `msg`, and `fn` when inside a function.

#### Scenario: DIA-001 Diagnostic shape
- **WHEN** `aval check` runs on a file with one unknown name
- **THEN** stdout contains exactly one line, which parses as a JSON object with keys `code`, `at`, `msg`, `fn`

#### Scenario: DIA-002 Clean file prints nothing
- **WHEN** `aval check` runs on a valid file
- **THEN** stdout is empty and the exit code is 0

### Requirement: Deterministic ordering
Diagnostics SHALL be ordered by position, then by code, and identical runs SHALL produce identical output.

#### Scenario: DIA-003 Several errors in order
- **WHEN** a file has an `E101` at line 5 and an `E301` at line 3
- **THEN** the `E301` line is printed before the `E101` line

### Requirement: Error recovery
After an error in one function, checking SHALL continue with the other functions.

#### Scenario: DIA-004 Errors in two functions
- **WHEN** two functions each contain one unknown name
- **THEN** `aval check` reports two `E101` diagnostics

### Requirement: Exit codes
Exit code SHALL be 0 when there are no errors (warnings allowed), 1 when there is at least one `E` or `T` diagnostic, and 2 on usage or I/O errors.

#### Scenario: DIA-005 Missing file
- **WHEN** `aval check` is given a path that does not exist
- **THEN** the exit code is 2

#### Scenario: DIA-006 Warning only
- **WHEN** the only diagnostic is a warning
- **THEN** the exit code is 0
