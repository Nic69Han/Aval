# stdlib Specification

## Purpose
Defines the prelude methods of the standard library: their names (borrowed from Rust), types,
declared costs and runtime faults.

## Requirements

### Requirement: List predicates and combinators
`List<T>` SHALL provide `is_empty() -> Bool`, `any(f) -> Bool`, `all(f) -> Bool` (with `f: T -> Bool`), `take(n) -> List<T>`, `skip(n) -> List<T>` (saturating, like Rust iterators), `concat(ys) -> List<T>`, and `dedup() -> List<T>` (removes consecutive duplicates).

#### Scenario: STD-001 any and all
- **WHEN** `fn f(xs: List<Int>) -> Bool` returns `xs.any(|x| x > 2) and not xs.all(|x| x > 2)` with example `[1, 3] => true`
- **THEN** `aval test` reports no failure

#### Scenario: STD-002 take and skip saturate
- **WHEN** `fn f(xs: List<Int>) -> List<Int>` returns `xs.take(10).skip(1)` with example `[1, 2, 3] => [2, 3]`
- **THEN** `aval test` reports no failure

#### Scenario: STD-003 dedup removes consecutive duplicates only
- **WHEN** `fn f(xs: List<Int>) -> List<Int>` returns `xs.dedup()` with example `[1, 1, 2, 1] => [1, 2, 1]`
- **THEN** `aval test` reports no failure

#### Scenario: STD-004 is_empty
- **WHEN** `fn f(xs: List<Int>) -> Bool` returns `xs.is_empty()` with examples `[] => true` and `[0] => false`
- **THEN** `aval test` reports no failure

### Requirement: Product of numbers
`List<Int>` and `List<Float>` SHALL provide `product()`, equal to 1 (or 1.0) for an empty list; Int overflow is a fault.

#### Scenario: STD-005 product
- **WHEN** `fn f(xs: List<Int>) -> Int` returns `xs.product()` with examples `[2, 3, 4] => 24` and `[] => 1`
- **THEN** `aval test` reports no failure

### Requirement: Numeric methods
`Int` SHALL provide `min(o)`, `max(o)`, `pow(e)` (`e >= 0`, overflow is a fault) and `to_string()`; `Float` SHALL provide `sqrt()`, `floor()`, `ceil()`, `round()`, `min(o)`, `max(o)`, `int()` (truncation toward zero, out-of-range is `overflow`) and `to_string()`.

#### Scenario: STD-006 Int helpers
- **WHEN** `fn f(a: Int, b: Int) -> Int` returns `a.min(b) + a.max(b).pow(2)` with example `2, 3 => 11`
- **THEN** `aval test` reports no failure

#### Scenario: STD-007 Float helpers
- **WHEN** `fn f(x: Float) -> Int` returns `x.sqrt().floor().int() + x.round().int()` with example `10.0 => 13`
- **THEN** `aval test` reports no failure

#### Scenario: STD-008 Negative exponent
- **WHEN** `fn f(a: Int) -> Int` has example `2 => 0` and returns `a.pow(-1)`
- **THEN** `aval test` reports `T700`

### Requirement: String operations
`String` SHALL support `+` (concatenation), `== != < <= > >=` (byte-wise order), and the methods `chars() -> List<String>`, `contains(t)`, `starts_with(t)`, `ends_with(t)`, `split(sep) -> List<String>`, `trim()`, `to_uppercase()`, `to_lowercase()` and `parse_int() -> Result<Int, String>`; `List<String>` SHALL provide `join(sep) -> String`.

#### Scenario: STD-009 Reverse words
- **WHEN** `fn f(s: String) -> String` returns `s.split(" ").reverse().join(" ")` with example `"a bc d" => "d bc a"`
- **THEN** `aval test` reports no failure

#### Scenario: STD-010 chars and concatenation
- **WHEN** `fn f(s: String) -> String` returns `s.chars().reverse().join("") + "!"` with example `"ab" => "ba!"`
- **THEN** `aval test` reports no failure

#### Scenario: STD-011 parse_int
- **WHEN** `fn f(s: String) -> Result<Int, String>` returns `s.trim().parse_int()` with examples `" 42 " => Ok(42)` and `"x" => Err("invalid digit")`
- **THEN** `aval test` reports no failure

#### Scenario: STD-012 String ordering and case
- **WHEN** `fn f(a: String, b: String) -> Bool` returns `a.to_lowercase() < b.to_uppercase().to_lowercase()` with example `"Abc", "abd" => true`
- **THEN** `aval test` reports no failure

### Requirement: Mixed-type string concatenation is rejected
`+` between `String` and a non-`String` SHALL be `E301`; conversion uses `to_string()`.

#### Scenario: STD-013 String plus Int
- **WHEN** `fn f(n: Int) -> String` returns `"n=" + n`
- **THEN** `aval check` reports `E301`

#### Scenario: STD-014 to_string
- **WHEN** `fn f(n: Int) -> String` returns `"n=" + n.to_string()` with example `7 => "n=7"`
- **THEN** `aval test` reports no failure

### Requirement: to_string on every scalar
`to_string()` SHALL be implemented for `Int`, `Float` (Rust `Display`), `Bool`, `Byte` and `String` (identity).

#### Scenario: STD-015 to_string on all receivers
- **WHEN** `fn f(s: String, b: Bool, y: Byte, x: Float) -> String` returns `s.to_string() + b.to_string() + y.to_string() + x.to_string()` with example `"a", true, 7, 2.5 => "atrue72.5"`
- **THEN** `aval test` reports no failure

### Requirement: Range methods
`Range<Int>` (`a..b`, `a..=b`) SHALL provide `len`, `is_empty`, `sum`, `product`, `min`, `max`, `contains(x)`, `count(x)`, `map(f)`, `filter(f)`, `fold(init, f)`, `any(f)`, `all(f)`, `reverse()`, `take(n)` and `skip(n)` with the types and semantics of the `List<Int>` methods of the same name, applied to the integers of the range in increasing order; `len` and `contains` SHALL be O(1).

#### Scenario: STD-016 Fold over an exclusive range
- **WHEN** `fn f(n: Int) -> Int` with `requires n >= 0 and n <= 1000` returns `(0..n).fold(0, |acc, i| acc + i)` with examples `5 => 10` and `0 => 0`
- **THEN** `aval test` reports no failure

#### Scenario: STD-017 Sum over an inclusive range
- **WHEN** `fn f(n: Int) -> Int` with `requires n >= 0 and n <= 1000` returns `(1..=n).sum()` with examples `4 => 10` and `0 => 0`
- **THEN** `aval test` reports no failure

#### Scenario: STD-018 len, is_empty and contains
- **WHEN** `fn f(a: Int, b: Int, x: Int) -> Bool` returns `(a..b).len() == 3 and not (a..b).is_empty() and (a..b).contains(x)` with examples `2, 5, 4 => true` and `2, 5, 5 => false`
- **THEN** `aval test` reports no failure

#### Scenario: STD-019 map and filter produce lists
- **WHEN** `fn f(n: Int) -> List<Int>` with `requires n >= 0 and n <= 1000` returns `(1..=n).filter(|i| i % 2 == 0).map(|i| i * i)` with example `6 => [4, 16, 36]`
- **THEN** `aval test` reports no failure

#### Scenario: STD-020 product, any, all, count, reverse
- **WHEN** `fn f(n: Int) -> Int` with `requires n >= 1 and n <= 15` returns `(1..=n).product() + (if (1..=n).any(|i| i > 3) { 1 } else { 0 }) + (if (1..=n).all(|i| i > 0) { 1 } else { 0 }) + (1..=n).count(2) + (1..=n).reverse()[0]`, with example `5 => 128`
- **THEN** `aval test` reports no failure

#### Scenario: STD-021 Empty ranges
- **WHEN** `fn f(a: Int, b: Int) -> Int` with `requires a >= b` returns `(a..b).sum() + (a..b).product() + (a..b).len()` with example `5, 2 => 1`
- **THEN** `aval test` reports no failure

#### Scenario: STD-022 min of an empty range faults
- **WHEN** `fn f(n: Int) -> Int` with `requires n >= 0 and n <= 5` returns `(3..n).min()` with example `3 => 0`
- **THEN** `aval test` reports `T700` with `fault` = `empty_list`

#### Scenario: STD-023 Unknown range member lists the available methods
- **WHEN** `fn f(n: Int) -> Int` returns `(0..n).average()`
- **THEN** `aval check` reports `E204` with `on` = `Range<Int>` and `avail` containing `fold`, `sum`, `len` and `map`

### Requirement: Lazy range reductions respect the step budget
Reducing range methods (`fold`, `sum`, `product`, `min`, `max`, `count`, `any`, `all`) SHALL NOT materialise the range and SHALL consume one evaluation step per element.

#### Scenario: STD-024 Huge range ends with step_limit, not an abort
- **WHEN** `fn f(n: Int) -> Int` returns `(0..n).sum()` with example `1000000000 => 0`
- **THEN** `aval test` reports `T700` with `fault` = `step_limit` and exits with code 1

### Requirement: Length overflow
`len` of a range whose length exceeds `i64::MAX` SHALL fault `overflow`.

#### Scenario: STD-025 Range longer than i64::MAX
- **WHEN** `fn f(a: Int, b: Int) -> Int` returns `(a..b).len()` with example `-9223372036854775807, 9223372036854775807 => 0`
- **THEN** `aval test` reports `T700` with `fault` = `overflow`

### Requirement: Range method arguments are typed
Arguments of range methods SHALL have the types of the corresponding list methods (`contains` and `count` take an `Int`).

#### Scenario: STD-026 contains with a non-Int
- **WHEN** `fn f(n: Int) -> Bool` returns `(0..n).contains(true)`
- **THEN** `aval check` reports `E301`

### Requirement: List-building methods never abort on huge ranges
`map`, `filter`, `reverse`, `take` and `skip` on a range SHALL fault `memory_limit` whenever the result would exceed 10,000,000 elements, for every range length including exactly 2^64.

#### Scenario: STD-027 Range of exactly 2^64 elements
- **WHEN** `fn f(a: Int, b: Int) -> Int` returns `((a - 1)..=b).skip(0).len()` with example `-9223372036854775807, 9223372036854775807 => 0`
- **THEN** `aval test` reports `T700` with `fault` = `memory_limit` and the process does not abort

#### Scenario: STD-028 Every reduction ends on a huge range
- **WHEN** each of `(0..n).min()`, `(0..n).max()`, `(0..n).count(5)`, `(0..n).any(|i| i < 0)`, `(0..n).product()` and `(0..n).map(|i| i).len()` is evaluated with `n = 1000000000`
- **THEN** each reports `T700` with `fault` = `step_limit` (reductions) or `memory_limit` (`map`) within a few seconds
