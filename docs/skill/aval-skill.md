# Aval — reference for agents (v0.1 MVP)

Aval is a small, Rust-like language whose compiler verifies your code against a locked spec. Write the
signature and spec first, then the body. Run `aval check FILE` (fast: syntax, types, effects, spec
rules) then `aval test FILE` (examples, property tests, oracle). Diagnostics are one JSON object per line;
fix them in order. Exit code 0 = accepted.

## Rules that differ from Rust

- Every file starts with `module name`. Statements end at a newline; no `;`. Indentation is ignored.
- Values/functions/fields are `snake_case`; types/variants are `PascalCase`. No shadowing.
- `let` always has a type: `let n: Int = xs.len()`. Everything is immutable. No `while`, no `var`, no
  `null`, no exceptions, no early `return`.
- A function body is `{ let* return expr }`: exactly one `return`, last. Inside `if`/`else`/`for`
  blocks the value is the last expression (no `return` there).
- `if` is an expression and always has `else`. `for x in xs { e }` returns a list. Accumulate with
  `xs.fold(init, |acc, x| ...)`.
- No implicit conversion: `Int / Float` is an error; use `.float()`. Integer literals become `Byte`
  when a `Byte` is expected. Logic is `and`, `or`, `not`. Ranges: `a..b` excludes `b`, `a..=b` includes it; a range has the list methods `len is_empty contains sum product min max count map filter fold any all reverse take skip` (`(0..n).fold(0, |a, i| a + i)`). `not` also works after `==`/`!=`: `a == not b`.
- Errors: return `Result<T, E>`, build with `Ok(v)` / `Err(e)`, propagate with `?`. `match` must be
  exhaustive (or use `_`).

## Function shape (clauses in this exact order, all optional unless `pub`)

```
fn name(p: Type, ...) -> Type
  requires <Bool>          # precondition, repeatable
  ensures <Bool>           # postcondition, repeatable; `result` = return value
  examples                 # one per line: inputs => output
    <in>, <in> => <out>
  oracle <fn_name>         # naive reference implementation, same signature
  effects none | a.b, c.d  # absent = pure
  alloc none | reuse(p)
  cost O(<expr over sizes>)
  latency < 2ms
{
  return <expr>
}
```

A `pub` function must have at least one `ensures` or `examples`, and a `cost`. Prefer properties that do
not depend on the implementation: for a mean, `approx(result * xs.len().float(), xs.sum())` beats
`result >= xs.min()`. Compare floats with `approx(a, b)`, not `==`.

## Example

```aval
module stats

pub fn mean(xs: List<Float>) -> Float
  requires xs.len() > 0
  ensures approx(result * xs.len().float(), xs.sum())
  examples
    [1.0, 2.0, 3.0] => 2.0
  cost O(xs.len())
{
  return xs.sum() / xs.len().float()
}

enum Shape { Circle(Float), Square(Float) }

fn area(s: Shape) -> Float
  examples
    Square(2.0) => 4.0
{
  return match s {
    Circle(r) => 3.14159 * r * r
    Square(c) => c * c
  }
}

pub fn half(x: Int) -> Result<Int, String>
  ensures result.is_err() or result.unwrap() * 2 == x
  cost O(1)
{
  return if x % 2 == 0 { Ok(x / 2) } else { Err("odd") }
}
```

## Types and declarations

`Int` (i64, overflow = fault), `Float` (f64), `Bool`, `String`, `Byte` (u8), `List<T>`,
`Result<T, E>`. `type Point { x: Int, y: Int }` → `Point { x: 1, y: 2 }`, `p.x`.
`enum Color { Red, Green, Blue }`, variants with payload `Circle(Float)`. Struct literals are not allowed
directly in an `if`/`match`/`for` head: parenthesise them.

## Standard library (pure unless noted)

Rust names and meanings. Methods return new values; nothing mutates.
- List<T>: `xs[i]`, `xs[a..b]`, `xs[a..=b]` (same line as `xs`; out of range = fault `index_out_of_bounds`),
  `len() is_empty() reverse() dedup() take(n) skip(n) concat(ys) push(x) contains(x) count(x) any(f) all(f)
  map(f) filter(f) fold(init, f)`; ordered T: `sort() min() max()`; Int/Float: `sum() product()`;
  `List<String>`: `join(sep)`. `min`/`max` of `[]` is a fault. `take`/`skip` saturate.
- Int: `float() abs() min(b) max(b) pow(e) to_string()`. Float: `abs() sqrt() floor() ceil() round()
  min(b) max(b) int()` (truncates) `to_string()`. Byte: `int() float()`. Bool/Byte: `to_string()`.
- String: `a + b`, `< <= > >=`, `len()` (bytes), `is_empty() chars()` (list of 1-char Strings) `contains(t)
  starts_with(t) ends_with(t) split(sep) trim() to_uppercase() to_lowercase() parse_int()`
  (`Result<Int, String>`). No indexing (use `chars()`); `"n=" + n` is an error, use `n.to_string()`.
- `approx(a, b)` (`approx`, `print`, `Ok`, `Err` cannot be redefined). Result: `is_ok() is_err() unwrap()`. `print(s)` has `effects io.stdout`.
- Lambdas: `|x| x * 2`, `|acc, x| acc + x`.

## Effects

A function may call an effectful function only if it declares the same effects. `par for` bodies must be
pure. Spec clauses must be pure.

## Spec lock

Humans run `aval spec lock FILE`, which records a hash of each `pub` signature + spec in `aval.lock`.
The hash also covers helpers called from `requires`/`ensures` and the oracle. Never edit `aval.lock` or
weaken a locked spec: `E900` means you changed it; restore it and fix the body.

## Grammar (EBNF, NL = newline)

```
module   = "module" path NL { import } { item }
item     = [ "pub" ] ( fn_decl | type_decl | enum_decl )
fn_decl  = "fn" ident "(" [ param { "," param } ] ")" "->" type NL spec guarantees fn_body
spec     = { "requires" expr NL } { "ensures" expr NL }
           [ "examples" NL { [ expr { "," expr } ] "=>" expr NL } ] [ "oracle" ident NL ]
guarantees = [ "effects" ( "none" | path { "," path } ) NL ] [ "alloc" ( "none" | "reuse" "(" ident ")" ) NL ]
           [ "cost" "O" "(" expr ")" NL ] [ "latency" "<" duration NL ]
fn_body  = "{" { let } ( "return" expr NL | "using" ident fn_body ) "}"
let      = "let" ident ":" type "=" expr NL
block    = "{" { let } expr "}"
expr     = or/and/not, then == != (right side may start with not) < <= > >= in, then .. ..=, then + -, then * / %, unary -, postfix .m(args) [i] [a..b] ?
primary  = literal | ident [ args ] | Ident [ args | "{" field: expr, ... "}" ] | "(" expr ")"
         | "[" exprs "]" | "if" expr block "else" ( block | if ) | "match" expr "{" { pattern "=>" expr NL } "}"
         | [ "par" ] "for" ident "in" expr block | "|" ident { "," ident } "|" expr | "result"
pattern  = "_" | literal | ident | Ident [ "(" pattern { "," pattern } ")" ]
type     = Ident [ "<" type { "," type } ">" ]
```

## Keywords (46)

fn let return type enum module import pub if else match for in par requires ensures result examples
oracle effects alloc cost latency reuse none resource using unsafe true false and or not Int Float Bool
String Byte List Set Map Queue Image Tensor Result Arena

## Diagnostics

Shape: `{"code":"E301","at":[line,col],"fn":"f","msg":"...", ...fields}`.

| Code | Meaning | Fields |
| --- | --- | --- |
| E001 | lexical error | |
| E002 | syntax error | expected |
| E101 | unknown name | name |
| E102 | duplicate name or shadowing | |
| E103 | unresolved import (MVP: single file) | module |
| E105 | reserved name (`O`) | |
| E106 | `result` outside `ensures` | |
| E107 | oracle is the function itself or impure | |
| E204 | unknown member | found, on, avail |
| E301 | type mismatch | expected, found |
| E302 | literal out of range | |
| E303 | wrong number of arguments | |
| E310 | non-exhaustive match | missing |
| E312 | uninhabited (infinitely recursive) type | |
| E313 | cannot infer type of `[]`: annotate with a `let` | |
| E420 | undeclared effect | effect |
| E421 | impure `par for` body | |
| E510 | `pub` fn without ensures/examples | |
| E511 | `pub` fn without cost | |
| E900 | locked spec modified | |
| W210 | `==` on Float in ensures | |
| W901 | `pub` fn not in aval.lock | |
| T100 | example failed | input, expected, got |
| T200 | ensures violated (shrunk) | violated, counterexample, got |
| T210 | `requires` rejects >90% of generated inputs | accepted, requested, reason (rejected/budget) |
| T600 | differs from oracle | counterexample, got, oracle |
| T700 | runtime fault: overflow, division_by_zero, unwrap_on_err, empty_list, index_out_of_bounds, memory_limit, stack_limit, step_limit | fault, counterexample |

`aval test FILE [--cases N] [--seed S] [--max-steps N]`: `step_limit` is 10M steps per input; recursion deeper than 10,000 calls is `stack_limit`.

Workflow: fix `E` codes first (they block `aval test`), then `T` codes. A `T200` counterexample is
shrunk (small, locally minimal): reproduce it mentally, fix the body, never the spec.
