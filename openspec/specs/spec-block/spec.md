# spec-block Specification

## Purpose
Defines the static rules of the spec block: public functions must carry a spec and a cost, `result`
is only meaningful in postconditions, and spec clauses are boolean.

## Requirements

### Requirement: Public functions declare a spec
Every `pub` function SHALL declare at least one `ensures` or an `examples` block; otherwise `E510` is reported.

#### Scenario: SPC-001 Pub function without spec
- **WHEN** `pub fn f(x: Int) -> Int` declares `cost O(1)` and nothing else
- **THEN** `aval check` reports `E510`

#### Scenario: SPC-002 Private function without spec
- **WHEN** `fn f(x: Int) -> Int` has no clauses
- **THEN** `aval check` reports no error

#### Scenario: SPC-003 Examples alone satisfy the rule
- **WHEN** `pub fn f(x: Int) -> Int` declares only `examples` and `cost O(1)`
- **THEN** `aval check` reports no `E510`

### Requirement: Public functions declare a cost
Every `pub` function SHALL declare a `cost` clause; otherwise `E511` is reported.

#### Scenario: SPC-004 Pub function without cost
- **WHEN** `pub fn f(x: Int) -> Int` declares `ensures result == x` and no cost
- **THEN** `aval check` reports `E511`

### Requirement: result only in ensures
The keyword `result` SHALL only appear inside `ensures`; elsewhere `E106` is reported.

#### Scenario: SPC-005 result in requires
- **WHEN** a function declares `requires result > 0`
- **THEN** `aval check` reports `E106`

#### Scenario: SPC-006 result in body
- **WHEN** a function body returns `result`
- **THEN** `aval check` reports `E106`

### Requirement: Spec clauses are Bool
`requires` and `ensures` expressions SHALL have type `Bool`.

#### Scenario: SPC-007 Non-Bool ensures
- **WHEN** a function declares `ensures result + 1`
- **THEN** `aval check` reports `E301`

### Requirement: Examples are type-checked
Each example's inputs SHALL match the parameter types and its output the return type.

#### Scenario: SPC-008 Example with wrong arity
- **WHEN** `fn f(x: Int, y: Int) -> Int` has example `1 => 2`
- **THEN** `aval check` reports `E303`

### Requirement: Oracle signature
An `oracle` SHALL name a function of the module with exactly the same parameter and return types.

#### Scenario: SPC-009 Oracle with different signature
- **WHEN** `fn f(x: Int) -> Int` declares `oracle g` where `fn g(x: Float) -> Int`
- **THEN** `aval check` reports `E301`

### Requirement: Float equality warning in ensures
Using `==` between `Float` values inside `ensures` SHALL produce warning `W210`.

#### Scenario: SPC-010 Float equality in ensures
- **WHEN** a function returning `Float` declares `ensures result == 1.0`
- **THEN** `aval check` reports `W210` and exits with code 0

### Requirement: Examples are closed expressions
Example inputs and outputs SHALL only use literals, constructors and prelude functions; parameter names and other module functions are not in scope (`E101`).

#### Scenario: SPC-011 Parameter name used in an example
- **WHEN** `fn f(x: Int) -> Int` has example `x => 99`
- **THEN** `aval check` reports `E101` with `name` = `x`

### Requirement: Oracles are pure and distinct
An `oracle` SHALL name a pure function other than the function itself; otherwise `E107` is reported.

#### Scenario: SPC-012 Oracle naming itself
- **WHEN** `fn f(x: Int) -> Int` declares `oracle f`
- **THEN** `aval check` reports `E107`

#### Scenario: SPC-013 Oracle with effects
- **WHEN** `fn f(x: Int) -> Int` declares `oracle g` and `g` declares `effects io.stdout`
- **THEN** `aval check` reports `E107`
