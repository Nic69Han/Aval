# self-test Specification

## Purpose
Defines `aval test`: execution of examples, property-based testing of `ensures` under `requires`,
oracle comparison and runtime faults, each failure shrunk to a minimal counterexample.

## Requirements

### Requirement: Self-test runs only on checked code
`aval test` SHALL first run all `aval check` stages and SHALL NOT execute anything if an error was found.

#### Scenario: TST-001 Type error blocks tests
- **WHEN** `aval test` runs on a file containing an `E301`
- **THEN** only the `E301` diagnostic is printed and the exit code is 1

### Requirement: Examples are executed
Each example SHALL be evaluated; a mismatch SHALL produce `T100` with fields `input`, `expected`, `got`. `Float` results are compared with `approx`.

#### Scenario: TST-002 Failing example
- **WHEN** `fn double(x: Int) -> Int` returns `x + 2` with example `3 => 6`
- **THEN** `aval test` reports `T100` with `expected` = 6 and `got` = 5

#### Scenario: TST-003 Float example uses approx
- **WHEN** `fn third() -> Float` returns `1.0 / 3.0` with example `=> 0.3333333333333333`
- **THEN** `aval test` reports no failure

### Requirement: Examples also check ensures
Each example input SHALL also be checked against every `ensures` clause.

#### Scenario: TST-004 Example satisfies output but violates ensures
- **WHEN** a function with `ensures result > 10` returns `x` and has example `5 => 5`
- **THEN** `aval test` reports `T200` with `counterexample` = `{"x": 5}`

### Requirement: Property testing of ensures
For each function with `ensures`, `aval test` SHALL generate inputs (default 1000 per function, deterministic seed) satisfying `requires`, including edge cases (0, 1, -1, extremes — also inside lists, fields and payloads —, empty, singleton, all-equal and long lists up to at least 32 elements), and report a violation as `T200` with fields `violated` (clause source text), `counterexample` (object keyed by parameter name) and `got`.

#### Scenario: TST-005 Buggy mean is caught
- **WHEN** `fn mean(xs: List<Float>) -> Float` with `requires xs.len() > 0` and `ensures approx(result * xs.len().float(), xs.sum())` returns `xs.sum() / 2.0`
- **THEN** `aval test` reports `T200` whose `violated` is `approx(result * xs.len().float(), xs.sum())`

#### Scenario: TST-006 Correct mean passes
- **WHEN** the same function returns `xs.sum() / xs.len().float()`
- **THEN** `aval test` reports no failure and exits with code 0

#### Scenario: TST-007 Requires filters inputs
- **WHEN** `fn inv(x: Int) -> Int` with `requires x != 0` and `ensures result == 100 / x` returns `100 / x`
- **THEN** `aval test` reports no `T700`

#### Scenario: TST-008 Deterministic runs
- **WHEN** `aval test` is run twice on the same failing file
- **THEN** both runs print identical output

### Requirement: Minimal counterexamples
A failing input SHALL be shrunk before reporting: integers toward 0, floats toward 0.0 and integral values, lists by removing elements then shrinking elements.

#### Scenario: TST-009 Shrinking an Int counterexample
- **WHEN** `fn f(x: Int) -> Int` with `ensures result < 100` returns `x`
- **THEN** `aval test` reports `T200` with `counterexample` = `{"x": 100}`

#### Scenario: TST-010 Shrinking a list counterexample
- **WHEN** `fn f(xs: List<Int>) -> Int` with `ensures result != 7` returns `xs.len()`
- **THEN** `aval test` reports `T200` with `counterexample` = `{"xs": [0, 0, 0, 0, 0, 0, 0]}`

### Requirement: Oracle comparison
When `oracle g` is declared, each generated input SHALL also be evaluated with `g`; a difference produces `T600` with `counterexample`, `got` and `oracle`.

#### Scenario: TST-011 Oracle disagreement
- **WHEN** `fn f(xs: List<Int>) -> Int` returns `xs.len()` with `oracle g`, where `g` returns `xs.sum()`
- **THEN** `aval test` reports `T600`

### Requirement: Runtime faults
Integer overflow, division by zero, `unwrap` on `Err` and `min`/`max` on an empty list SHALL stop evaluation of that input and produce `T700` with `fault` (`overflow`, `division_by_zero`, `unwrap_on_err`, `empty_list`, `step_limit`) and `counterexample`.

#### Scenario: TST-012 Division by zero
- **WHEN** `fn inv(x: Int) -> Int` with `ensures true` returns `100 / x` and no `requires`
- **THEN** `aval test` reports `T700` with `fault` = `division_by_zero` and `counterexample` = `{"x": 0}`

#### Scenario: TST-013 Overflow
- **WHEN** `fn sq(x: Int) -> Int` with `ensures result >= 0` returns `x * x`
- **THEN** `aval test` reports `T700` with `fault` = `overflow`

### Requirement: One failure per function
`aval test` SHALL report at most one `T` diagnostic per function per kind, and continue with other functions.

#### Scenario: TST-014 Two failing functions
- **WHEN** two functions each have a failing example
- **THEN** `aval test` reports two `T100` diagnostics

### Requirement: Extreme values inside compound inputs
Generated lists, struct fields and variant payloads SHALL include extreme scalar values, so overflow bugs on compound inputs are found.

#### Scenario: TST-015 Overflow inside a list
- **WHEN** `fn f(xs: List<Int>) -> List<Int>` with `ensures result.len() == xs.len()` returns `xs.map(|x| x.abs())`
- **THEN** `aval test` reports `T700` with `fault` = `overflow`

### Requirement: Vacuous properties are failures
When fewer than 10% of the requested generated cases satisfy `requires` (and at least 10 were requested), `aval test` SHALL report `T210` with fields `accepted` and `requested` instead of passing silently.

#### Scenario: TST-016 Unsatisfiable requires
- **WHEN** `fn f(x: Int) -> Int` with `requires x == 4242` and `ensures result == x` returns `x`
- **THEN** `aval test` reports `T210` and exits with code 1

#### Scenario: TST-017 Long-list requires is satisfiable
- **WHEN** `fn f(xs: List<Int>) -> Int` with `requires xs.len() > 8` and `ensures result == xs.len()` returns `xs.len()`
- **THEN** `aval test` reports no failure

### Requirement: Faulting examples are reported
An example whose inputs, expected output or evaluation fault SHALL be reported as `T700` with `fault` and `input`, never skipped.

#### Scenario: TST-018 Expected value faults
- **WHEN** `fn f(xs: List<Int>) -> Int` has example `[1] => [].min()`
- **THEN** `aval test` reports `T700` with `fault` = `empty_list`

### Requirement: Evaluation step budget
Each evaluation of one input SHALL be limited to 10,000,000 evaluation steps; exceeding it SHALL produce `T700` with `fault` = `step_limit`, so `aval test` always terminates quickly.

#### Scenario: TST-019 Runaway recursion
- **WHEN** `fn fib(n: Int) -> Int` with `requires n >= 0` and `ensures result >= 0` returns `if n < 2 { n } else { fib(n - 1) + fib(n - 2) }`
- **THEN** `aval test` reports `T700` with `fault` = `step_limit` and finishes in under 60 seconds

### Requirement: Recursive input types
Generation of recursive types SHALL be depth-limited and choose base variants at the limit, so `aval test` never crashes.

#### Scenario: TST-020 Recursive type parameter
- **WHEN** `type T { l: L }`, `enum L { Cons(T), Nil }` and `fn f(t: T) -> Bool` with `ensures result` returns `true`
- **THEN** `aval test` reports no failure and exits with code 0

### Requirement: Shrinking recursive values reduces size
Shrinking SHALL prefer smaller constructions (variants of lower construction rank, e.g. `Nil` before `Cons`), so a recursive counterexample is no larger than needed.

#### Scenario: TST-021 Recursive list counterexample is small
- **WHEN** `enum L { Cons(Int, L), Nil }` and `fn len(l: L) -> Int` with `ensures result < 3` returns the correct length
- **THEN** `aval test` reports `T200` with `counterexample` = `{"l": {"Cons": [0, {"Cons": [0, {"Cons": [0, "Nil"]}]}]}}` and `got` = 3

### Requirement: Budgets never hide a vacuous property
`T210` SHALL be reported whenever fewer than 10% of the requested cases were accepted, including when the step budget stopped generation early; a `T200` whose shrinking was cut by the budget SHALL carry `"shrunk": false`.

#### Scenario: TST-022 Expensive requires that rejects everything
- **WHEN** `fn f(x: Int) -> Int` has `requires slow(x) and x == 4242` where `slow` builds a 1,000,000-element list and returns `true`, with `ensures result == x`, body `x + 1`
- **THEN** `aval test` reports `T210` and exits with code 1

### Requirement: Vacuous preconditions on parameterless functions
For a function without parameters, a `requires` that evaluates to `false` SHALL be reported as `T210` with `accepted` = 0 and `requested` = 1.

#### Scenario: TST-023 requires false without parameters
- **WHEN** `fn f() -> Int` has `requires false` and `ensures result == 1` and returns `2`
- **THEN** `aval test` reports `T210`

### Requirement: Out-of-range index fault
An index or slice outside the bounds of the receiver SHALL stop evaluation of that input with `T700` and `fault` = `index_out_of_bounds`.

#### Scenario: TST-024 Index past the end
- **WHEN** `fn f(xs: List<Int>) -> Int` with `ensures true` returns `xs[0]`
- **THEN** `aval test` reports `T700` with `fault` = `index_out_of_bounds` and `counterexample` = `{"xs": []}`

#### Scenario: TST-025 Inverted slice bounds
- **WHEN** `fn f(xs: List<Int>) -> List<Int>` has example `[1, 2, 3] => [1]` and returns `xs[2..1]`
- **THEN** `aval test` reports `T700` with `fault` = `index_out_of_bounds`

### Requirement: Memory budget
A `String` or `List` value larger than 10,000,000 bytes or elements SHALL stop evaluation of that input with `T700` and `fault` = `memory_limit`; `aval test` SHALL never abort.

#### Scenario: TST-026 Exponential string growth
- **WHEN** `fn f(xs: List<Int>) -> Int` with `ensures result >= 0` returns `xs.fold("a", |acc, x| acc + acc).len()`
- **THEN** `aval test` reports `T700` with `fault` = `memory_limit` and exits with code 1

### Requirement: Deep recursion
The interpreter SHALL support a recursion depth of at least 10,000 nested user-function calls; deeper recursion is a fault `T700` with `fault` = `stack_limit`.

#### Scenario: TST-027 Natural recursion to depth 5,000
- **WHEN** `fn count(n: Int) -> Int` with `requires n >= 0 and n <= 5000` and `ensures result == n` returns `if n == 0 { 0 } else { count(n - 1) + 1 }`, with example `5000 => 5000`
- **THEN** `aval test` reports no failure

#### Scenario: TST-028 Unbounded recursion
- **WHEN** `fn f(n: Int) -> Int` has example `1 => 0` and returns `f(n + 1)`
- **THEN** `aval test` reports `T700` with `fault` = `stack_limit` or `step_limit`
