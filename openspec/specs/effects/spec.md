# effects Specification

## Purpose
Defines static effect checking: functions are pure unless they declare effects, effectful calls must be
declared by the caller, and `par for` bodies must be pure.

## Requirements

### Requirement: Undeclared effects are rejected
A function calling a function with effects SHALL declare each of those effects in its own `effects` clause; otherwise `E420` is reported with field `effect`.

#### Scenario: EFF-001 Print without effects clause
- **WHEN** `fn greet() -> Bool` returns `print("hi")`
- **THEN** `aval check` reports `E420` with `effect` = `io.stdout`

#### Scenario: EFF-002 Declared effect accepted
- **WHEN** `fn greet() -> Bool` declares `effects io.stdout` and returns `print("hi")`
- **THEN** `aval check` reports no error

#### Scenario: EFF-003 Effects propagate through user functions
- **WHEN** `fn a() -> Bool` declares `effects io.stdout` and calls `print`, and `fn b() -> Bool` without effects returns `a()`
- **THEN** `aval check` reports `E420` in `b`

### Requirement: effects none is explicit purity
`effects none` SHALL mean the same as an absent clause.

#### Scenario: EFF-004 effects none with print
- **WHEN** a function declares `effects none` and calls `print("x")`
- **THEN** `aval check` reports `E420`

### Requirement: Parallel loops require pure bodies
The body of a `par for` SHALL be pure; otherwise `E421` is reported.

#### Scenario: EFF-005 Impure par for
- **WHEN** a function declaring `effects io.stdout` returns `par for x in xs { print("x") }`
- **THEN** `aval check` reports `E421`

#### Scenario: EFF-006 Pure par for
- **WHEN** a function returns `par for x in xs { x * 2 }` with example `[1, 2] => [2, 4]`
- **THEN** `aval test` reports no failure

### Requirement: Spec clauses are pure
`requires`, `ensures` and `examples` expressions SHALL be pure.

#### Scenario: EFF-007 Effect in ensures
- **WHEN** a function declaring `effects io.stdout` has `ensures print("x")`
- **THEN** `aval check` reports `E420`
