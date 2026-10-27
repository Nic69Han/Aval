# syntax Specification

## Purpose
Defines the grammar of Aval modules, functions and expressions, including the rules that keep one
canonical way to write each construct, and the syntax errors returned to agents.

## Requirements

### Requirement: Module header
Every source file SHALL start with `module <path>`.

#### Scenario: SYN-001 Missing module header
- **WHEN** a file starts directly with `fn f() -> Int`
- **THEN** `aval check` reports `E002` at line 1 with `expected` containing `module`

### Requirement: Syntax errors list expected tokens
A syntax error SHALL be reported as `E002` with an `expected` array listing the tokens or clauses allowed at that point.

#### Scenario: SYN-002 Missing arrow
- **WHEN** a function is declared as `fn f(x: Int) Int`
- **THEN** `aval check` reports `E002` whose `expected` contains `->`

### Requirement: Fixed clause order
Spec and guarantee clauses SHALL appear in the order `requires*`, `ensures*`, `examples?`, `oracle?`, `effects?`, `alloc?`, `cost?`, `latency?`.

#### Scenario: SYN-003 Canonical order accepted
- **WHEN** a function declares `requires`, `ensures`, `examples`, `effects`, `cost`, `latency` in that order
- **THEN** `aval check` reports no syntax error

#### Scenario: SYN-004 Latency before cost rejected
- **WHEN** a function declares `latency < 2ms` followed by `cost O(1)`
- **THEN** `aval check` reports `E002` at the `cost` clause

#### Scenario: SYN-005 Ensures before requires rejected
- **WHEN** a function declares `ensures result > 0` followed by `requires x > 0`
- **THEN** `aval check` reports `E002` at the `requires` clause

### Requirement: Function body terminal
A function body SHALL end with exactly one terminal: `return expr`, or a `using` block ending with a terminal; no statement may follow it.

#### Scenario: SYN-006 Missing return
- **WHEN** a function body contains only `let y: Int = 1`
- **THEN** `aval check` reports `E002` with `expected` containing `return`

#### Scenario: SYN-007 Statement after return
- **WHEN** a function body contains `return 1` followed by `let y: Int = 2`
- **THEN** `aval check` reports `E002` at the `let`

### Requirement: Expression blocks yield their last expression
`if`/`else`, `for` and block bodies SHALL evaluate to their final expression and SHALL NOT contain `return`.

#### Scenario: SYN-008 If block value
- **WHEN** a function returns `if x > 0 { let y: Int = x * 2` newline `y } else { 0 }` and is tested with example `3 => 6`
- **THEN** `aval test` reports no failure

#### Scenario: SYN-009 Return inside an if block
- **WHEN** a function body contains `return if x > 0 { return 1 } else { 0 }`
- **THEN** `aval check` reports `E002` at the inner `return`

### Requirement: If requires else
Every `if` SHALL have an `else` branch.

#### Scenario: SYN-010 If without else
- **WHEN** a function returns `if x > 0 { 1 }`
- **THEN** `aval check` reports `E002` with `expected` containing `else`

### Requirement: Let requires a type annotation
Every `let` SHALL carry a type annotation.

#### Scenario: SYN-011 Let without type
- **WHEN** a function body contains `let y = 1`
- **THEN** `aval check` reports `E002` with `expected` containing `:`

### Requirement: Operator precedence
Binary operators SHALL follow the precedence of the grammar: `or` < `and` < `not` < comparison < range < `+ -` < `* / %` < unary `-` < postfix.

#### Scenario: SYN-012 Multiplication binds tighter than addition
- **WHEN** a function returns `1 + 2 * 3` with example `=> 7` (no parameters)
- **THEN** `aval test` reports no failure

#### Scenario: SYN-013 Comparisons do not chain
- **WHEN** a function returns `1 < 2 < 3`
- **THEN** `aval check` reports `E002`

### Requirement: Struct literals, enums and patterns
The parser SHALL accept `type` and `enum` declarations, struct literals `Name { f: e }`, and `match` patterns (`_`, literals, bindings, variants with sub-patterns).

#### Scenario: SYN-014 Struct literal and field access
- **WHEN** a module declares `type Point { x: Int, y: Int }` and a function returns `Point { x: 1, y: 2 }.y` with example `=> 2`
- **THEN** `aval test` reports no failure

#### Scenario: SYN-015 No struct literal in if condition
- **WHEN** a function returns `if Point { x: 1, y: 2 }.x > 0 { 1 } else { 0 }`
- **THEN** `aval check` reports `E002`

### Requirement: Lambdas and for expressions
The parser SHALL accept lambdas `|a, b| expr` as call arguments and `[par] for x in e { ... }` expressions.

#### Scenario: SYN-016 Fold with a lambda
- **WHEN** a function returns `xs.fold(0, |acc, x| acc + x)` with example `[1, 2, 3] => 6`
- **THEN** `aval test` reports no failure

#### Scenario: SYN-017 For produces a list
- **WHEN** a function returns `for x in xs { x * 2 }` with example `[1, 2] => [2, 4]`
- **THEN** `aval test` reports no failure

### Requirement: Cost clause syntax
The `cost` clause SHALL be written `cost O(expr)`, and `O` SHALL be accepted only there.

#### Scenario: SYN-018 Cost without O
- **WHEN** a function declares `cost xs.len()`
- **THEN** `aval check` reports `E002` with `expected` containing `O`

### Requirement: Index and slice expressions
A postfix `[i]` on the same line as its receiver SHALL index a `List<T>` (type `T`); `[a..b]` and `[a..=b]` SHALL slice a `List<T>` (type `List<T>`) with Rust bounds semantics. A `String` is not indexable (`E204`); use `chars()`.

#### Scenario: SYN-019 List index
- **WHEN** `fn f(xs: List<Int>) -> Int` returns `xs[1]` with example `[5, 6, 7] => 6`
- **THEN** `aval test` reports no failure

#### Scenario: SYN-020 List slice exclusive and inclusive
- **WHEN** `fn f(xs: List<Int>) -> List<Int>` returns `xs[1..3].concat(xs[0..=0])` with example `[5, 6, 7, 8] => [6, 7, 5]`
- **THEN** `aval test` reports no failure

#### Scenario: SYN-021 Bracket after a newline is a list literal
- **WHEN** an `examples` block has the line `[1, 2] => 3` directly after the line `[0] => 0`
- **THEN** `aval check` parses two examples and reports no error

#### Scenario: SYN-022 Index must be Int
- **WHEN** `fn f(xs: List<Int>) -> Int` returns `xs[1.0]`
- **THEN** `aval check` reports `E301`

#### Scenario: SYN-023 Indexing a String is not allowed
- **WHEN** `fn f(s: String) -> String` returns `s[0]`
- **THEN** `aval check` reports `E204`

### Requirement: not as right operand of equality
The right operand of `==` and `!=` MAY be preceded by `not`; `a == not b` SHALL mean `a == (not b)`. A leading `not` keeps its meaning: `not a == b` is `not (a == b)`. `not` is accepted in no other operand position.

#### Scenario: SYN-024 Equality with a negated operand
- **WHEN** `fn f(a: Bool, b: Bool) -> Bool` returns `a == not b` with examples `true, false => true` and `true, true => false`
- **THEN** `aval test` reports no failure

#### Scenario: SYN-025 The negation binds to the operand only
- **WHEN** `fn f(x: Bool, y: Bool, z: Bool) -> Bool` returns `x == not y and z` with examples `true, false, true => true` and `true, false, false => false`
- **THEN** `aval test` reports no failure

#### Scenario: SYN-026 Leading not is unchanged
- **WHEN** `fn f(a: Int, b: Int) -> Bool` returns `not a == b` with examples `1, 2 => true` and `2, 2 => false`
- **THEN** `aval test` reports no failure

#### Scenario: SYN-027 Other positions and non-Bool operands
- **WHEN** `fn f(a: Int) -> Int` returns `a + not true`, and in another function `fn g(a: Bool) -> Bool` returns `a == not 3`
- **THEN** `aval check` reports `E002` for `f` and `E301` for `g`

### Requirement: Only one not, no chained comparison
After `==` or `!=` the parser SHALL accept at most one `not`, and comparisons SHALL still not chain.

#### Scenario: SYN-028 Double not and chained equality
- **WHEN** a function returns `a == not not b`, and another function returns `a == not b == c`
- **THEN** `aval check` reports `E002` for each
