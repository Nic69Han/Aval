# types Specification

## Purpose
Defines static, local, strong typing for the MVP types (`Int`, `Float`, `Bool`, `String`, `Byte`,
`List<T>`, `Result<T, E>`, user `type` and `enum`), with no implicit conversion.

## Requirements

### Requirement: Return and let types are checked
The checker SHALL report `E301` when an expression's type differs from the expected type (return type, `let` annotation, argument, condition).

#### Scenario: TYP-001 Returning Float from an Int function
- **WHEN** `fn f() -> Int` returns `1.0`
- **THEN** `aval check` reports `E301` with fields `expected` = `Int` and `found` = `Float`

#### Scenario: TYP-002 Let annotation mismatch
- **WHEN** a body contains `let s: String = 3`
- **THEN** `aval check` reports `E301`

#### Scenario: TYP-003 Non-Bool if condition
- **WHEN** a function returns `if 1 { 2 } else { 3 }`
- **THEN** `aval check` reports `E301` with `expected` = `Bool`

### Requirement: No implicit numeric conversion
Arithmetic and comparison operands SHALL have the same numeric type; conversion uses `.float()` or `.int()`.

#### Scenario: TYP-004 Float divided by Int
- **WHEN** `fn f(xs: List<Float>) -> Float` returns `xs.sum() / xs.len()`
- **THEN** `aval check` reports `E301`

#### Scenario: TYP-005 Explicit conversion accepted
- **WHEN** `fn f(xs: List<Float>) -> Float` returns `xs.sum() / xs.len().float()`
- **THEN** `aval check` reports no error

### Requirement: Integer literals take the expected type
An integer literal SHALL have type `Int` unless the expected type is `Byte`; a `Byte` literal outside `0..=255` is `E302`.

#### Scenario: TYP-006 Byte literal from expected type
- **WHEN** `fn f(p: Byte) -> Byte` returns `if p > 128 { 255 } else { 0 }`
- **THEN** `aval check` reports no error

#### Scenario: TYP-007 Byte literal out of range
- **WHEN** `fn f() -> Byte` returns `256`
- **THEN** `aval check` reports `E302`

### Requirement: Unknown members list alternatives
Calling an unknown method or field SHALL report `E204` with fields `found`, `on` (receiver type) and `avail` (sorted list of available members).

#### Scenario: TYP-008 Unknown list method
- **WHEN** `fn f(xs: List<Float>) -> Float` returns `xs.average()`
- **THEN** `aval check` reports `E204` with `on` = `List<Float>` and `avail` containing `len`, `sum`, `min`, `max`

### Requirement: Call arity
Calls SHALL pass exactly the declared number of arguments.

#### Scenario: TYP-009 Too many arguments
- **WHEN** `fn g(x: Int) -> Int` is called as `g(1, 2)`
- **THEN** `aval check` reports `E303`

### Requirement: Result and the question mark
`Ok(v)` and `Err(e)` SHALL construct `Result` values; `?` SHALL only be used on a `Result` inside a function returning a `Result` with the same error type.

#### Scenario: TYP-010 Question mark propagates
- **WHEN** `fn half(x: Int) -> Result<Int, String>` returns `if x % 2 == 0 { Ok(x / 2) } else { Err("odd") }`, and `fn quarter(x: Int) -> Result<Int, String>` returns `Ok(half(half(x)?)?)`, with example `8 => Ok(2)` on `quarter`
- **THEN** `aval test` reports no failure

#### Scenario: TYP-011 Question mark in a non-Result function
- **WHEN** `fn f(x: Int) -> Int` returns `half(x)?`
- **THEN** `aval check` reports `E301`

### Requirement: Exhaustive match
A `match` SHALL cover every variant of an enum or `Result`, or contain `_`; missing variants are listed in field `missing` of `E310`.

#### Scenario: TYP-012 Missing variant
- **WHEN** `enum Color { Red, Green, Blue }` is matched with arms for `Red` and `Green` only
- **THEN** `aval check` reports `E310` with `missing` = [`Blue`]

#### Scenario: TYP-013 Match on enum with payload
- **WHEN** `enum Shape { Circle(Float), Square(Float) }` and `fn area(s: Shape) -> Float` returns a match with both arms, with example `Square(2.0) => 4.0`
- **THEN** `aval test` reports no failure

### Requirement: List element types are uniform
All elements of a list literal SHALL have the same type.

#### Scenario: TYP-014 Mixed list
- **WHEN** a function returns `[1, 2.0]`
- **THEN** `aval check` reports `E301`

### Requirement: Lambda parameter inference
Lambda parameter types SHALL be inferred from the expected function type of the callee.

#### Scenario: TYP-015 Map with lambda
- **WHEN** `fn f(xs: List<Int>) -> List<Bool>` returns `xs.map(|x| x > 0)`
- **THEN** `aval check` reports no error

### Requirement: Empty list literals take their type from context
An empty list literal SHALL get its element type from the expected type or from the first later use that fixes it; it SHALL NOT behave as a list of any type.

#### Scenario: TYP-016 Empty list cannot absorb mismatched elements
- **WHEN** `fn f(x: Int) -> List<String>` returns `[].push(x)`
- **THEN** `aval check` reports `E301`

#### Scenario: TYP-017 Fold accumulator from empty list is checked
- **WHEN** `fn f(xs: List<Int>) -> List<String>` returns `xs.fold([], |acc, x| acc.push(x))`
- **THEN** `aval check` reports `E301`

### Requirement: Fields are not methods
Accessing a struct field with call syntax SHALL report `E204`.

#### Scenario: TYP-018 Field called as a method
- **WHEN** `type Point { x: Int, y: Int }` and `fn f(p: Point) -> Int` returns `p.x()`
- **THEN** `aval check` reports `E204` with `found` = `p.x`

### Requirement: Uninhabited types are rejected
A `type` or `enum` with no finite value (every construction path recurses into itself) SHALL be reported as `E312`.

#### Scenario: TYP-019 Self-recursive struct
- **WHEN** a module declares `type N { n: N }`
- **THEN** `aval check` reports `E312`

#### Scenario: TYP-020 Recursive enum with a base case is accepted
- **WHEN** a module declares `type T { l: L }` and `enum L { Cons(T), Nil }`
- **THEN** `aval check` reports no error

### Requirement: Whitespace inside type expressions
Whitespace and newlines inside `< >` of a type expression SHALL be insignificant.

#### Scenario: TYP-021 Space before generic arguments
- **WHEN** a parameter is declared `xs: List <Int>`
- **THEN** `aval check` reports no error

### Requirement: Empty list element types are inferred soundly
The element type of an empty list literal SHALL be an inference variable unified at every use (branches of `if`/`match`, lambda results, nested lists, method receivers); a variable still unresolved at the end of the function SHALL default to the first type that fixes it or be reported as `E313` when nothing fixes it and a method needs it.

#### Scenario: TYP-022 Empty list in an if branch
- **WHEN** `fn f(x: Int) -> List<String>` returns `(if x > 0 { [] } else { [x] }).reverse()`
- **THEN** `aval check` reports `E301`

#### Scenario: TYP-023 Fold returning a new list literal
- **WHEN** `fn f(xs: List<Int>) -> List<String>` returns `xs.fold([], |acc, x| [x])`
- **THEN** `aval check` reports `E301`

#### Scenario: TYP-024 Nested empty list
- **WHEN** `fn f(x: Int) -> List<List<String>>` returns `[[]].push([x])`
- **THEN** `aval check` reports `E301`

#### Scenario: TYP-025 Generic type in a return position with spaces
- **WHEN** `fn f(xs: List <Int>) -> List <Int>` returns `xs`
- **THEN** `aval check` reports no error

### Requirement: Uninferable empty list
When nothing fixes the element type of an empty list and a method requires it, the checker SHALL report `E313`.

#### Scenario: TYP-026 Sum of an unconstrained empty list
- **WHEN** `fn f() -> Bool` returns `[].sum() == [].sum()`
- **THEN** `aval check` reports `E313`

### Requirement: Fold accumulators are typed by fixpoint
The lambda of `fold` SHALL be re-checked with the refined accumulator type until it no longer changes; an accumulator whose element type is still unknown when a method or a typed position uses it SHALL be reported as `E313`.

#### Scenario: TYP-027 Accumulator used with a wrong element type
- **WHEN** `fn f(xs: List<Int>) -> List<Int>` returns `xs.fold([], |acc, x| acc.map(|y| y.len()).push(x))`
- **THEN** `aval check` reports `E204` or `E301` and exits with code 1

#### Scenario: TYP-028 Accumulator passed to a typed function
- **WHEN** `fn g(s: List<String>) -> List<String>` returns `s`, and `fn f(xs: List<Int>) -> List<Int>` returns `xs.fold([], |acc, x| g(acc).push(x))`
- **THEN** `aval check` reports `E301`
