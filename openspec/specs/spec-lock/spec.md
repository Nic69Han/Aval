# spec-lock Specification

## Purpose
Locks the spec block of public functions with a hash recorded in `aval.lock`, so that an agent writing
the body cannot weaken the spec without a human approval visible in the repository.

## Requirements

### Requirement: Lock file generation
`aval spec lock <file>` SHALL write `aval.lock` next to the source file, with one line `<file name>:<module>.<fn> sha256:<64 hex>` per `pub` function, sorted.

#### Scenario: LCK-001 Lock writes one line per pub function
- **WHEN** `aval spec lock` runs on a module `m` with `pub fn a` and `pub fn b` and private `fn c`
- **THEN** `aval.lock` contains exactly the lines for `m.a` and `m.b`, in that order

### Requirement: Hash normalisation
The hash SHALL be computed over the token sequence of the function signature and spec clauses (`requires`, `ensures`, `examples`, `oracle`), ignoring whitespace and comments.

#### Scenario: LCK-002 Reformatting does not change the hash
- **WHEN** a locked function's spec block gets extra spaces and a comment
- **THEN** `aval check` reports no `E900`

#### Scenario: LCK-003 Body change does not change the hash
- **WHEN** only the body of a locked function changes
- **THEN** `aval check` reports no `E900`

### Requirement: Modified spec detection
`aval check` SHALL report `E900` with field `fn` for every locked function whose current hash differs from `aval.lock`.

#### Scenario: LCK-004 Weakened ensures
- **WHEN** a locked function's `ensures result > x` is changed to `ensures true`
- **THEN** `aval check` reports `E900` and exits with code 1

### Requirement: Unlocked public functions
When `aval.lock` exists, a `pub` function absent from it SHALL produce warning `W901`; when no lock file exists, no lock diagnostic is produced.

#### Scenario: LCK-005 New pub function not in lock
- **WHEN** a `pub` function is added to a module whose `aval.lock` predates it
- **THEN** `aval check` reports `W901` and exits with code 0

#### Scenario: LCK-006 No lock file
- **WHEN** a module with `pub` functions has no `aval.lock`
- **THEN** `aval check` reports no `E900` and no `W901`

### Requirement: Lock covers the functions the spec depends on
The hash of a function SHALL also cover the full source (signature, clauses and body) of every module function reachable from its spec clauses, including the oracle and the helpers called in `requires`/`ensures`, transitively.

#### Scenario: LCK-007 Weakened ensures helper
- **WHEN** a locked function has `ensures ok(x, result)` and only the body of helper `ok` is changed to return `true`
- **THEN** `aval check` reports `E900` for the locked function

#### Scenario: LCK-008 Changed oracle body
- **WHEN** only the body of the oracle of a locked function changes
- **THEN** `aval check` reports `E900` for the locked function

### Requirement: Stale lock entries
Every `aval.lock` entry for the checked module SHALL match an existing `pub` function; an entry whose function was removed or renamed SHALL be reported as `E900` with field `reason` = `missing`.

#### Scenario: LCK-009 Locked function deleted
- **WHEN** a locked `pub` function is deleted from the module
- **THEN** `aval check` reports `E900` with `reason` = `missing`

#### Scenario: LCK-010 Locked function renamed
- **WHEN** a locked `pub fn a` is renamed `pub fn b` with the same spec
- **THEN** `aval check` reports `E900` with `reason` = `missing` and `W901` for `b`

### Requirement: Unambiguous hash input
The hashed token stream SHALL encode each token's kind and length so that no two distinct token sequences hash the same input.

#### Scenario: LCK-011 String literal cannot mimic tokens
- **WHEN** a locked `ensures result == "a"` is changed so that the string literal contains the text of other tokens (e.g. `"a\" or true or \"b"`)
- **THEN** `aval check` reports `E900`

### Requirement: Module rename does not escape the lock
Lock entries are keyed by source file name; an entry for the checked file whose module or function no longer exists SHALL be reported as `E900` with `reason` = `missing`.

#### Scenario: LCK-012 Module renamed and spec weakened
- **WHEN** a locked file's header is changed from `module m` to `module m2` and an `ensures` is weakened
- **THEN** `aval check` reports `E900` with `reason` = `missing`

### Requirement: Lock covers types used by the spec
The hash SHALL also cover the declarations of user types and enums reachable from the signature and from the hashed functions.

#### Scenario: LCK-013 Field type change
- **WHEN** a locked `pub fn f(p: P) -> Int` uses `type P { v: Int }` and the declaration changes to `type P { v: Byte }`
- **THEN** `aval check` reports `E900`

### Requirement: File rename does not escape the lock
Checking any file of a directory SHALL report `E900` with `reason` = `missing` for every `aval.lock` entry whose source file no longer exists in that directory.

#### Scenario: LCK-014 Source file renamed and spec weakened
- **WHEN** a locked `main.aval` is renamed `main2.aval` and an `ensures` is weakened, and `aval check main2.aval` runs
- **THEN** `aval check` reports `E900` with `reason` = `missing` and exits with code 1
