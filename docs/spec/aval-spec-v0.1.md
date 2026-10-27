# Aval — Language Specification v0.1.1

Status: **draft, normative for the MVP**. Canonical language: English.

This document is the human-readable reference. The testable acceptance criteria live in
[`openspec/specs/`](../../openspec/specs) (each scenario has a stable ID such as `SYN-004`), and the
executable conformance tests live in [`tests/conformance/`](../../tests/conformance). The three layers
are kept separate and linked by IDs; see [`docs/process/traceability.md`](../process/traceability.md).

---

## 1. Goals and principles

Aval is a programming language designed to be **written by AI agents and verified by a compiler** that
gives its *aval* (French for "approval", "go-ahead") to the code. The aim: code that compiles on the
first attempt, does what its spec says, does not leak, and stays within a declared cost.

1. **One canonical way to write each thing.** Few keywords, no competing syntactic sugar.
2. **Everything an agent needs is in the signature.** Types, contract, effects, allocation and cost are
   visible without reading the body or other files.
3. **Grammar and types are designed for constrained decoding.** An invalid or ill-typed token can be
   masked before it is emitted.
4. **The spec is an independent source of truth.** It is written before the body, then locked against
   the agent.
5. **Diagnostics are an API.** Compact JSON errors, minimal counterexamples.
6. **Cost is visible and checked.** Complexity, allocation and latency are part of the contract.
7. **Keywords chosen for tokenizers.** Every keyword is a single token on the target tokenizers
   (measured by experiment E1, see `experiments/e1-tokenizers/`).

Non-goals for v0.1: human ergonomics, inheritance, exceptions, reflection, metaprogramming.

## 2. Keywords

v0.1 freezes **46 reserved words**. An identifier may not reuse any of them.

| Category | Words | Role |
| --- | --- | --- |
| Structure | `fn` `let` `return` `type` `enum` `module` `import` `pub` | Functions, immutable bindings, types, modules, explicit imports |
| Control | `if` `else` `match` `for` `in` `par` | Condition, pattern matching, loop over a collection, parallelism over pure code |
| Specification | `requires` `ensures` `result` `examples` `oracle` | Locked spec block |
| Guarantees | `effects` `alloc` `cost` `latency` `reuse` `none` | Effects, allocation, complexity, time budget |
| Resources | `resource` `using` `unsafe` | Linear types, guaranteed release, audited FFI |
| Values & operators | `true` `false` `and` `or` `not` | Booleans and logic spelled out |
| Scalar types | `Int` `Float` `Bool` `String` `Byte` | Base types |
| Collections | `List` `Set` `Map` `Queue` | Collections; `Queue` is always bounded |
| Compute & memory | `Image` `Tensor` `Result` `Arena` | Dimensions in the type, errors, graphs |

Deliberately excluded: `while`, `null`, `try`/`catch`/`throw`, `class`, inheritance, `goto`, `var`.
`in` serves both loops (`for x in xs`) and membership/range tests (`result in a..b`).

**Contextual words (not reserved).** `O` is meaningful only directly after `cost` (see §8); declaring a
type named `O` is an error (`E105`). `Ok`, `Err` are the variants of `Result` and are ordinary
PascalCase variant names. `_` is the wildcard pattern.

## 3. Understanding without training

No model trained up to September 2026 can have seen Aval, born in October 2026, so the language must be
understood with its reference in context and no fine-tuning. Rule: **every construct borrows the syntax
and meaning of a language these models know well, and no known symbol is reused with a different
meaning.**

| Aval construct | Borrowed from | Status |
| --- | --- | --- |
| `fn`, `let`, `->`, `match` … `=>`, `\|x\| expr`, `if`/`else` as expression, block value = last expression, `Result`, `Ok`/`Err`, `?`, `unwrap`, `is_ok`/`is_err` | Rust | Conforming |
| `requires`, `ensures`, `result` | Dafny, Verus, Prusti | Conforming |
| `using` to release a resource | C# | Conforming |
| `effects` | Koka (effect types) | Conforming |
| `module`, `import` | Python, Kotlin | Conforming |
| `Int`, `Float`, `Bool`, `List`, `Set`, `Map` | Python, Kotlin, Swift | Conforming |
| `String` (draft said `Text`) | Rust, Swift, Java, C# | Fixed |
| `approx(a, b)` (draft said `~=`; `~=` means "not equal" in Lua) | — | Fixed |
| `a..b` exclusive, `a..=b` inclusive | Rust | Fixed |
| `examples` | Python doctests, Pyret `where` | New, explicit |
| `alloc`, `cost`, `latency`, `resource`, `oracle`, `par for` | no direct equivalent | New: transparent English words, defined in the reference |

- **Compact reference.** `aval skill print` prints the full reference; target ≤ 4,000 tokens
  (o200k), checked in CI.
- **Novelty is confined to signature clauses.** A function body reads like simplified Rust.

## 4. Lexical structure

- **Encoding**: UTF-8. Identifiers are ASCII.
- **Case is significant and carries meaning**: `snake_case` identifiers (`ident`, regex
  `[a-z_][a-z0-9_]*`) name values, functions, fields and effects; `PascalCase` identifiers (`Ident`,
  regex `[A-Z][A-Za-z0-9]*`) name types, enum variants and dimension variables. A lone `_` is the
  wildcard.
- **Comments**: `#` to end of line.
- **Literals**: integers (`0`, `42`; no sign — `-` is an operator), floats (`2.0`, `1.5e3`; a dot with
  digits on both sides is required), strings in double quotes with escapes `\" \\ \n \t`, booleans
  `true`/`false`, durations (`2ms`, `500us`, `1s`, `10ns`).
- **Newlines**: a newline (`NL`) terminates a statement or a clause. Consecutive newlines and comment
  lines collapse into one `NL`. Newlines are **ignored** inside `( )` and `[ ]`, and after `{`, `,`,
  `=>`, `->`, `=`, and any binary operator — so long expressions can be split after an operator.
- **Indentation has no meaning.**
- Lexical errors: `E001` (invalid character, unterminated string, malformed number).

## 5. Formal grammar

The grammar is EBNF. It is exported verbatim to GBNF/Lark for constrained decoding (follow-up change
`add-gbnf-export`). Terminals in quotes; `NL` as defined in §4.

```
module      = "module" path NL { import } { item } ;
path        = ident { "." ident } ;
import      = "import" path [ "{" Ident { "," Ident } "}" ] NL ;
item        = [ "pub" ] ( fn_decl | type_decl | enum_decl | resource_decl ) ;

type_decl     = "type" Ident [ generics ] "{" field { "," field } "}" NL ;
enum_decl     = "enum" Ident [ generics ] "{" variant { "," variant } "}" NL ;
resource_decl = "resource" Ident [ "{" field { "," field } "}" ] NL ;
generics      = "<" Ident { "," Ident } ">" ;
field         = ident ":" type_expr ;
variant       = Ident [ "(" type_expr { "," type_expr } ")" ] ;

fn_decl     = "fn" ident "(" [ param { "," param } ] ")" "->" type_expr NL
              spec guarantees fn_body ;
param       = ident ":" type_expr ;

spec        = { "requires" expr NL } { "ensures" expr NL }
              [ "examples" NL example { example } ] [ "oracle" ident NL ] ;
example     = [ expr { "," expr } ] "=>" expr NL ;   (* inputs omitted for 0-parameter functions *)

guarantees  = [ "effects" ( "none" | effect { "," effect } ) NL ]
              [ "alloc" ( "none" | "reuse" "(" ident ")" ) NL ]
              [ "cost" "O" "(" expr ")" NL ]
              [ "latency" "<" duration NL ] ;
effect      = ident { "." ident } ;

fn_body     = "{" { let_stmt } terminal "}" NL ;
terminal    = "return" expr NL
            | "using" ident fn_body ;
let_stmt    = "let" ident ":" type_expr "=" expr NL ;
block       = "{" { let_stmt } expr [ NL ] "}" ;

expr        = or_expr ;
or_expr     = and_expr { "or" and_expr } ;
and_expr    = not_expr { "and" not_expr } ;
not_expr    = [ "not" ] cmp_expr ;
cmp_expr    = range_expr [ ( "==" | "!=" ) [ "not" ] range_expr
                         | ( "<" | "<=" | ">" | ">=" | "in" ) range_expr ] ;
range_expr  = add_expr [ ( ".." | "..=" ) add_expr ] ;
add_expr    = mul_expr { ( "+" | "-" ) mul_expr } ;
mul_expr    = unary { ( "*" | "/" | "%" ) unary } ;
unary       = [ "-" ] postfix ;
postfix     = primary { "." ident [ args ] | "[" expr "]" | "?" } ;
args        = "(" [ expr { "," expr } ] ")" ;
primary     = literal | "result" | "(" expr ")" | list_lit
            | ident [ args ]                      (* variable or call *)
            | Ident [ args | struct_lit ]         (* variant, dimension variable, struct *)
            | if_expr | match_expr | for_expr | lambda ;
struct_lit  = "{" ident ":" expr { "," ident ":" expr } "}" ;
if_expr     = "if" expr block "else" ( block | if_expr ) ;
match_expr  = "match" expr "{" { pattern "=>" expr NL } "}" ;
pattern     = "_" | literal | ident | Ident [ "(" pattern { "," pattern } ")" ] ;
for_expr    = [ "par" ] "for" ident "in" expr block ;
lambda      = "|" ident { "," ident } "|" expr ;
list_lit    = "[" [ expr { "," expr } ] "]" ;

type_expr   = Ident [ "<" type_arg { "," type_arg } ">" ] ;
type_arg    = type_expr | int | ident ":" expr ;
```

Syntactic rules not expressible in EBNF:

- **Fixed clause order.** Spec clauses then guarantee clauses appear exactly in the order of the grammar
  (`requires*`, `ensures*`, `examples?`, `oracle?`, `effects?`, `alloc?`, `cost?`, `latency?`). Any other
  order is a syntax error `E002` whose `expected` field lists the clauses allowed at that point.
- **Function bodies end with exactly one terminal**: `return expr`, or a `using` block whose own body
  ends with a terminal. There is no early `return`; early exit happens only through `?`.
- **Expression blocks** (`if`, `else`, `for`, lambda bodies written as blocks) evaluate to their final
  expression, as in Rust. `return` is not allowed inside them.
- **`if` always has an `else`**: it is an expression, never a statement.
- **No struct literal in condition position.** In the condition of `if`, the scrutinee of `match` and the
  collection of `for`, `Ident {` is not parsed as a struct literal (same rule as Rust). Parenthesise to
  use one.
- **`for` is an expression** producing the list of its body values; accumulation uses `fold` (§9).
- **The type annotation is mandatory in `let`**: it guides constrained decoding.
- **Index and slice.** In `postfix`, `[ expr ]` indexes a `List<T>` when `expr` is an `Int` and slices it
  when `expr` is a range (`xs[a..b]`, `xs[a..=b]`; both bounds are required and `Int`). The `[` must be on
  the same line as its receiver: after a newline it starts a list literal (so `examples` lines such as
  `[1, 2] => 3` stay separate). Inside `( )` and `[ ]` newlines are ignored (§4), so there a `[` on the
  next line indexes the previous line's expression. Out-of-range indexes and slices are the runtime fault
  `index_out_of_bounds`. A `String` is not indexable (`E204`); use `chars()`.
- **Calls are by name only**: `args` follows an `ident`, an `Ident` or a method name; lambdas are passed
  to functions, never called directly. Syntax errors: `E002`, with `expected` listing the allowed tokens.

## 6. Type system

Typing is static, strong and **purely local**: each function is checked against its own signature and
the signatures it calls, so a file is re-checked in milliseconds. Checking is bidirectional: the expected
type flows into literals, lambdas and empty lists.

- **Immutable by default.** No value can be changed after binding. "In-place" updates come from memory
  reuse (`alloc reuse`), not mutation.
- **No implicit conversion.** `Int` to `Float` goes through `.float()`; `Float / Int` is a type error
  (`E301`). Mixed arithmetic is always an error.
- **Literal typing.** An integer literal has type `Int`, unless the expected type is `Byte`, in which
  case it must fit in `0..=255` (`E302` otherwise). A float literal has type `Float`. This mirrors Rust.
- **Numeric semantics.** `Int` is a 64-bit signed integer; overflow and division by zero are runtime
  faults (`T700` under `aval test`). `Byte` is an unsigned 8-bit integer. `Float` is IEEE-754 binary64.
  `Int / Int` is truncating division, `%` is the remainder with the sign of the dividend (Rust).
- **Dimensions in types.** `Image<Byte, H, W>`, `Tensor<Float, 3, N>`: `H`, `W`, `N` are dimension
  variables, bound by the parameter types of the signature and checked at compile time. A dimension
  variable is also in scope **as an `Int` value** in the spec clauses and the body (e.g.
  `ensures result.len() == N`).
- **Bounded collections.** A collection that outlives a function call must declare a bound and a policy:
  `Queue<Frame, max: 64, full: drop_oldest>`.
- **Explicit errors.** A fallible function returns `Result<T, E>`, built with `Ok(v)` / `Err(e)`; `?`
  propagates the error; `match` must cover every variant (`E310` otherwise).
- **Linear types.** A value of a `resource` type must be consumed exactly once (§10).
- **Generics without overloading.** A function name has a single signature in a module (`E102`).
- **Equality.** `==` is structural and defined on every type except `Float`-containing types compared
  inside `ensures`, where `approx` is recommended (warning `W210`, not an error).

The type system is designed for constrained decoding: at each position, the checker can compute the set
of well-typed continuations, hence the allowed tokens.

## 7. Spec block and self-test

The spec block (`requires`, `ensures`, `examples`, `oracle`) is written before the body and **locked**:
the agent that writes the body cannot weaken it to make its code pass.

- **S1.** Every `pub` function declares at least one `ensures` or an `examples` block; otherwise
  compilation fails (`E510`). Every `pub` function declares a `cost` clause (`E511`).
- **S2.** `aval test` runs the `examples`, then generates 1,000 inputs per function
  (configurable), keeping only those that satisfy `requires`. The generator also tries adversarial
  inputs: empty, all equal, all distinct, already sorted, extreme values (also inside lists, fields and
  payloads). An `examples` result of type `Float` is compared with `approx`. If fewer than 10% of the
  generated cases satisfy `requires`, the property is vacuous and `T210` is reported. Example
  expressions are closed: they cannot name parameters.
- **S3.** The spec block is hashed. Any change raises `E900 spec modified`, cleared only by a human
  approval recorded in the repository (lock file `aval.lock`, §7.1).
- **S4.** An `oracle` (naive reference implementation with the same signature) is compared with the body
  on the same inputs (`T600`).
- **S5.** Every failure is shrunk to a minimal counterexample and emitted as compact JSON (§11).
- `result` is only valid inside `ensures` (`E106` elsewhere). `requires` and `ensures` must be `Bool`
  (`E301`).

Usage rule: a property that does not depend on the implementation beats an example. For a mean,
`result in xs.min()..=xs.max()` lets a bug through, whereas
`approx(result * xs.len().float(), xs.sum())` catches it.

### 7.1 Spec lock (`aval.lock`)

- `aval spec lock` (a command intended for humans) writes `aval.lock` next to the module: one line per
  `pub` function, `file.aval:module.fn sha256:<hex>`, the hash being computed on the normalized token stream (kind
  and length of each token) of the signature and spec block, plus the full source of every module
  function reachable from the spec clauses (helpers called in `requires`/`ensures`, the oracle),
  transitively, and the declarations of the user types they use. Whitespace and comments do not change the hash.
- `aval check` recomputes every hash. A locked function whose hash differs raises `E900`. A `pub`
  function absent from the lock raises warning `W901 spec not locked`. A lock entry whose function no
  longer exists (deleted or renamed) raises `E900` with `reason: "missing"`.
- Human approval = a reviewed commit that modifies `aval.lock`. The repository protects `aval.lock` with
  `CODEOWNERS` (human maintainers only) and branch protection, so an agent cannot approve its own spec.
  Signed commits are recommended.

## 8. Effects, allocation, cost and latency

Four optional clauses describe what a function may do and what it costs; their absence always has the
strictest or most explicit meaning.

| Clause | Values | If absent | Checking |
| --- | --- | --- | --- |
| `effects` | `none`, or a list (`io.camera`, `fs.read`, `io.stdout`…) | `none`: pure function | Static: calling an effectful function requires declaring the effect (`E420`) |
| `alloc` | `none`, `reuse(x)` | Free allocation | Static, plus allocator instrumentation (`E430`) |
| `cost` | `O(expr)` over input sizes | Mandatory for `pub` (`E511`) | Static by cost composition, else empirical (`T400`) |
| `latency` | `< duration` | No budget | Measured on the target defined in `aval.toml` (`T500`) |

- **Purity and parallelism.** `par for` is accepted only if its body is pure (`E421`): parallelism is
  safe by construction, with no locks and no data races.
- **Cost expressions.** `O(...)` takes an arithmetic expression over input sizes (`xs.len()`, dimension
  variables, `log(n)`), e.g. `cost O(xs.len() * log(xs.len()))`, `cost O(H * W)`, `cost O(1)`.
- **Cost composition.** Every standard library function declares its cost (`List.contains`: O(n);
  `Set.contains`: O(1)). An O(n) loop calling an O(n) function gives O(n²), rejected if the declaration
  says O(n).
- **Empirical check.** When static analysis is inconclusive, the compiler counts the steps of each loop
  on inputs of growing size and keeps the model (1, log n, n, n log n, n², n³) that best explains the
  growth. Step counting is deterministic, unlike wall-clock time.
- **Latency.** Complexity ignores constant factors (cache, branch prediction); `latency` is therefore
  measured in real time, over hundreds of runs, on the target hardware.
- **`alloc reuse(x)`.** The function reuses the memory of `x` when `x` is uniquely referenced at the call
  site. The compiler checks this at each call site: when the argument is the caller's last use of a
  uniquely-owned value, no allocation happens; otherwise one copy is made and warning `W430 reuse
  defeated` is emitted. The guarantee is therefore "no allocation **when ownership is unique**", and the
  compiler tells you when it is not.

## 9. Standard library (MVP subset)

Prelude functions and methods available in v0.1 MVP, with their declared cost (n = length of the
receiver). Everything here is pure.

| Name | Signature | Cost |
| --- | --- | --- |
| `xs.len()` | `List<T> -> Int` | O(1) |
| `xs.sum()` | `List<Int> -> Int`, `List<Float> -> Float` | O(n) |
| `xs.min()`, `xs.max()` | `List<T> -> T` (`T` ordered); requires non-empty, else `T700` | O(n) |
| `xs.contains(x)` | `List<T>, T -> Bool` | O(n) |
| `xs.map(f)` | `List<T>, (T -> U) -> List<U>` | O(n · cost(f)) |
| `xs.filter(f)` | `List<T>, (T -> Bool) -> List<T>` | O(n · cost(f)) |
| `xs.fold(init, f)` | `List<T>, U, (U, T -> U) -> U` | O(n · cost(f)) |
| `xs.count(x)` | `List<T>, T -> Int` | O(n) |
| `xs.reverse()` | `List<T> -> List<T>` | O(n) |
| `xs.sort()` | `List<T> -> List<T>` (`T` ordered) | O(n log n) |
| `xs.push(x)` | `List<T>, T -> List<T>` (new list) | O(n) |
| `i.float()` | `Int -> Float`, `Byte -> Float` | O(1) |
| `b.int()` | `Byte -> Int` | O(1) |
| `x.abs()` | `Int -> Int`, `Float -> Float` | O(1) |
| `s.len()` | `String -> Int` (bytes) | O(1) |
| `xs[i]` | `List<T>, Int -> T`; `T700` `index_out_of_bounds` unless `0 <= i < len` | O(1) |
| `xs[a..b]`, `xs[a..=b]` | `List<T> -> List<T>`; `T700` `index_out_of_bounds` unless `0 <= a <= b <= len` (`..=`: `b < len`) | O(k) |
| `xs.is_empty()`, `s.is_empty()` | `List<T> -> Bool`, `String -> Bool` | O(1) |
| `xs.any(f)`, `xs.all(f)` | `List<T>, (T -> Bool) -> Bool` | O(n · cost(f)) |
| `xs.take(n)`, `xs.skip(n)` | `List<T>, Int -> List<T>` (saturating; `n < 0` counts as 0) | O(n) |
| `xs.concat(ys)` | `List<T>, List<T> -> List<T>` | O(n) |
| `xs.dedup()` | `List<T> -> List<T>` (removes consecutive duplicates) | O(n) |
| `xs.product()` | `List<Int> -> Int`, `List<Float> -> Float`; 1 / 1.0 when empty; Int overflow is `T700` | O(n) |
| `xs.join(sep)` | `List<String>, String -> String` | O(n) |
| `a.min(b)`, `a.max(b)` | `Int, Int -> Int`, `Float, Float -> Float` | O(1) |
| `a.pow(e)` | `Int, Int -> Int`; `e < 0` is `T700` `negative_exponent`, overflow is `T700` | O(log e) |
| `x.sqrt()`, `x.floor()`, `x.ceil()`, `x.round()` | `Float -> Float` (`round`: half away from zero) | O(1) |
| `x.int()` | `Float -> Int`, truncation toward zero; NaN or out of range is `T700` `overflow` | O(1) |
| `v.to_string()` | `Int`, `Float`, `Bool`, `Byte`, `String -> String` (Rust `Display`: `2.0` prints `2`) | O(1) |
| `a + b`, `a < b`, ... | `String` concatenation and byte-wise ordering; `String + Int` is `E301` | O(n) |
| `s.chars()` | `String -> List<String>` (one string per Unicode scalar value) | O(n) |
| `s.contains(t)`, `s.starts_with(t)`, `s.ends_with(t)` | `String, String -> Bool` | O(n) |
| `s.split(sep)` | `String, String -> List<String>` (Rust `str::split` semantics) | O(n) |
| `s.trim()`, `s.to_uppercase()`, `s.to_lowercase()` | `String -> String`; whitespace and case mapping follow the Unicode tables of the Rust toolchain that built `aval` (e.g. `"ß".to_uppercase()` is `"SS"`), so results for non-ASCII text can change with the toolchain's Unicode version | O(n) |
| `s.parse_int()` | `String -> Result<Int, String>`; errors `"invalid digit"`, `"empty string"`, `"number too large"`, `"number too small"` | O(n) |
| `approx(a, b)` | `Float, Float -> Bool`: `abs(a-b) <= 1e-9 * max(1, abs(a), abs(b))` | O(1) |
| `r.is_ok()`, `r.is_err()` | `Result<T, E> -> Bool` | O(1) |
| `r.unwrap()` | `Result<T, E> -> T`; `T700` on `Err` | O(1) |
| `print(s)` | `String -> Bool`, **effects `io.stdout`** | O(n) |

`a..b` and `a..=b` on `Int` produce a range usable with `in` and as a `for` collection.

A `Range<Int>` also has the following methods, with the types and semantics of the `List<Int>` methods of
the same name applied to its integers in increasing order. An empty range (`a >= b`, or `a > b` for `..=`)
gives `sum` 0, `product` 1, `any` false, `all` true, and `min`/`max` fault `empty_list`.

| Call | Type | Cost |
|---|---|---|
| `r.len()` | `Range<Int> -> Int`; `T700` `overflow` if longer than `i64::MAX` | O(1) |
| `r.is_empty()` | `Range<Int> -> Bool` | O(1) |
| `r.contains(x)` | `Range<Int>, Int -> Bool` | O(1) |
| `r.sum()`, `r.product()`, `r.min()`, `r.max()` | `Range<Int> -> Int` | O(n) |
| `r.count(x)` | `Range<Int>, Int -> Int` | O(n) |
| `r.any(f)`, `r.all(f)` | `Range<Int>, (Int -> Bool) -> Bool` | O(n * cost(f)) |
| `r.fold(init, f)` | `Range<Int>, U, (U, Int -> U) -> U` | O(n * cost(f)) |
| `r.map(f)` | `Range<Int>, (Int -> U) -> List<U>` | O(n * cost(f)) |
| `r.filter(f)` | `Range<Int>, (Int -> Bool) -> List<Int>` | O(n * cost(f)) |
| `r.reverse()`, `r.take(n)`, `r.skip(n)` | `-> List<Int>` | O(n) |

Reductions iterate lazily, one evaluation step per element (a huge range ends with `T700` `step_limit`);
methods that build a list are bounded by the memory limit (`memory_limit`).
A module cannot declare a function, type or variant named `approx`, `print`, `Ok` or `Err` (`E102`).
`Image` methods (`count`, `par_map`) belong to the `vision` library, outside the MVP.

## 10. Memory model and resources

Aval aims at **zero leaks by construction** for unreachable memory and for resources, and at bounded and
detected logical leaks.

| Layer | Mechanism | Guarantee |
| --- | --- | --- |
| Ordinary data | Precise reference counting (Perceus) over immutable data | No unreachable memory: immutability forbids cycles |
| Resources | Linear types (`resource`), `using` block | Every resource is consumed exactly once, checked at compile time (`E410`) |
| Logical growth | Bounded collections, heap measurement in self-test | Bounded by types, detected by tests (`T300`), not proven absent |

- **No borrow checker.** The agent never writes lifetimes (the main source of Rust compile errors).
- **In-place reuse.** When a value has a single reference, `alloc reuse(x)` reuses its memory (§8).
- **Cyclic structures** go through `Arena<T>`: nodes reference each other by index and the arena is freed
  in one block.
- **No exceptions.** Errors go through `Result`, so leaving a `using` block always releases the resource,
  including when `?` propagates an error.
- **FFI.** Every external call is marked `unsafe`, wrapped in a linear resource, and requires human
  review. The guarantees above hold only outside these calls.

## 11. Diagnostics and error codes

Every diagnostic is a **one-line JSON object**, stable across versions: `E` codes come from the compiler,
`W` codes are warnings, `T` codes come from the self-test. Common fields: `code`, `at` (`[line, column]`,
1-based), `fn` (when inside a function), `msg` (short English text). Code-specific fields are listed in
`docs/spec/diagnostics.schema.json` (follow-up).

| Code | Family | Example |
| --- | --- | --- |
| `E001` | Lexical | Invalid character, unterminated string |
| `E002` | Syntax | Unexpected token; field `expected` lists allowed tokens |
| `E101` | Unknown name | Undeclared identifier |
| `E102` | Duplicate | Two items with the same name in a module (no overloading) |
| `E103` | Unresolved module | `import` of an unknown module |
| `E105` | Reserved name | Type named `O`, identifier equal to a keyword |
| `E106` | Misplaced `result` | `result` outside `ensures` |
| `E107` | Invalid oracle | Oracle is the function itself or has effects |
| `E204` | Unknown member | Method that does not exist on this type, with the list of available ones |
| `E301` | Type mismatch | `Int` received where `Float` is expected |
| `E302` | Literal out of range | `256` where a `Byte` is expected |
| `E303` | Arity | Wrong number of arguments |
| `E310` | Non-exhaustive match | Missing variant, listed in field `missing` |
| `E313` | Cannot infer | Element type of an empty list cannot be inferred |
| `E312` | Uninhabited type | `type N { n: N }` has no finite value |
| `E410` | Resource | Resource not consumed at end of scope |
| `E420` | Effect | Effect used without being declared in `effects` |
| `E421` | Impure parallel body | `par for` body is not pure |
| `E430` | Allocation | Allocation in an `alloc none` function |
| `E510` | Missing spec | `pub` function without `ensures` nor `examples` |
| `E511` | Missing cost | `pub` function without `cost` |
| `E900` | Spec | Spec block modified without human approval |
| `W100` | Weak spec | Too many surviving mutants |
| `W210` | Float equality | `==` on `Float` in `ensures`; use `approx` |
| `W430` | Reuse defeated | `alloc reuse(x)` argument not uniquely owned at a call site |
| `W901` | Unlocked spec | `pub` function absent from `aval.lock` |
| `T100` | Example | Output differs from the expected one |
| `T200` | Property | `ensures` clause violated, minimal counterexample |
| `T210` | Vacuous property | Fewer than 10% of generated cases satisfy `requires` (fields `accepted`, `requested`, `reason`: `rejected` or `budget`) |
| `T300` | Memory | Heap did not return to its initial level |
| `T400` | Cost | Observed complexity above the declared one |
| `T500` | Latency | `latency` budget exceeded on target |
| `T600` | Oracle | Result differs from the reference implementation |
| `T700` | Runtime fault | Field `fault`: `overflow`, `division_by_zero`, `unwrap_on_err`, `empty_list`, `index_out_of_bounds`, `negative_exponent`, `memory_limit` (a `String` over 10,000,000 bytes or a `List` over 10,000,000 elements), `stack_limit` (more than 10,000 nested user calls), `step_limit` (10M steps per input by default; `aval test --max-steps N` changes it) |

Examples (the `T200` line is produced by a buggy body `return xs.sum() / 2.0` of `mean`, §14):

```
{"code":"T200","at":[4,11],"fn":"mean","msg":"ensures violated","violated":"approx(result * xs.len().float(), xs.sum())","counterexample":{"xs":[1.0]},"got":0.5}
{"code":"E204","at":[7,23],"fn":"mean","msg":"unknown member","found":"xs.average","on":"List<Float>","avail":["len","sum","min","max"],"fix":"xs.sum() / xs.len().float()"}
```

`fix` is optional: when the correction is certain, `aval fix` applies it without going back through the
agent. Measured on the `E204` example, a JSON diagnostic costs 49 tokens versus 106 for a rustc-style
message (o200k). Exit codes: `0` success (warnings allowed), `1` diagnostics with errors or failed tests,
`2` usage or internal error.

## 12. Compilation and the Rust target

Aval is a language in its own right whose v0.1 compiles to readable Rust: every check happens before
generation, and the agent never sees a Rust error.

1. **Parse**: lexer and parser from the grammar above.
2. **Resolve**: names and explicit imports.
3. **Types**: local inference, dimensions, resource linearity.
4. **Guarantees**: effects, allocation, cost composition.
5. **Spec**: hash check of the locked block (`E900`).
6. **Rust generation**: `Rc`/`Arc` with in-place reuse (`Rc::make_mut`), no `unsafe` outside FFI.
7. **Build**: `rustc` and LLVM produce the native binary.

- `aval check` runs steps 1–5 without calling `rustc`: the agent's fast loop.
- `aval test` runs the self-test (examples, properties, oracle; memory and cost later) on the reference
  interpreter.
- `aval build` runs the whole chain (after the MVP).
- `aval fix` applies certain corrections proposed by diagnostics (after the MVP).
- `aval skill print` prints the compact language reference embedded in the binary, so it can never
  diverge from the installed version.

If `rustc` rejects generated code, it is a bug in the Aval compiler, never an agent error. A dedicated
backend (Cranelift or direct LLVM) may replace Rust later without changing the language.

## 13. MVP scope

The MVP (OpenSpec change `add-aval-core-mvp`) delivers a **reference interpreter and checker** — the
fastest path to evaluating how well models write Aval:

- In: lexer, parser, name resolution, types for `Int Float Bool String Byte List Result` + user `type`
  and `enum`, `match` with exhaustiveness, effects (`E420`, `E421`), spec rules (`E510`, `E511`, `E106`),
  spec lock (`E900`, `W901`), interpreter, `aval test` with examples (`T100`), property testing with
  shrinking (`T200`), oracle (`T600`), runtime faults (`T700`), JSON diagnostics, `aval skill print`.
- Parsed and checked for well-formedness only: `alloc`, `cost`, `latency`, `par` (executed sequentially).
- Out (follow-up changes): `resource`/`using` linearity (`E410`), `Image`/`Tensor` and dimension
  checking, `Set`/`Map`/`Queue`/`Arena`, multi-file modules, `T300`/`T400`/`T500`, `W100` mutation,
  GBNF export, Rust backend, `aval fix`, LSP.

## 14. Complete example

This module gathers the v0.1 mechanisms: contract, purity, memory reuse, linear resource and latency
budget. (It uses the `vision` library, which is outside the MVP; `mean` alone is MVP-runnable.)

```
module inspection.line3
import vision.camera { Camera, Frame, CamError }
# In vision.camera:  pub type Frame { image: Image<Byte, 1080, 1920>, ts: Int }

pub fn mean(xs: List<Float>) -> Float
  requires xs.len() > 0
  ensures approx(result * xs.len().float(), xs.sum())
  examples
    [1.0, 2.0, 3.0] => 2.0
  cost O(xs.len())
{
  return xs.sum() / xs.len().float()
}

pub fn threshold(img: Image<Byte, H, W>, s: Byte) -> Image<Byte, H, W>
  ensures result.count(0) + result.count(255) == H * W
  alloc reuse(img)
  cost O(H * W)
{
  return img.par_map(|p| if p > s { 255 } else { 0 })
}

pub fn inspect(cam: Camera, s: Byte) -> Result<Int, CamError>
  ensures result.is_err() or result.unwrap() >= 0
  effects io.camera
  cost O(1)
  latency < 2ms
{
  using cam {
    let f: Frame = cam.grab()?
    let bin: Image<Byte, 1080, 1920> = threshold(f.image, s)
    return Ok(bin.count(255))
  }
}
```

What the compiler guarantees before any execution:

- `mean` and `threshold` are pure; `threshold` parallelises safely.
- In `threshold`, `255` and `0` are typed `Byte` by the expected element type; `H * W` uses the
  dimension variables as `Int` values.
- `threshold` allocates nothing in `inspect`, because `f.image` is the last use of a uniquely-owned
  value; elsewhere `W430` would say otherwise.
- The camera is always released, including when `grab()` fails and `?` propagates the error.
- `cost O(1)` holds in `inspect` because the image dimensions are constants (1080 × 1920).
- Any version of `threshold` producing a pixel that is neither black nor white is rejected by the
  self-test (`T200`), and any quadratic version by the cost check (`T400`).

## 15. Open questions for v0.2

Open — each becomes an OpenSpec change proposal when tackled:

- **Generic constraints.** Traits (`T: Ord`) to write generic `sort` or `max`? MVP: built-in only.
- **Concurrency.** Beyond `par for`: async and camera streams — bounded channels, actors, other?
- **Closed-API models.** Without logits there is no mask: define the fallback loop (JSON diagnostics and
  fast `aval check`).
- **Standard library v0.1.** Complete list with declared costs (`Set`, `Map`, `Image`, `String`).
- **Effects vs. capabilities.** Should `effects io.camera` be derived from holding a linear `Camera`?
  Does `cam.grab()` borrow or consume the resource inside `using`?
- **Spec lock scope.** Lock only `pub` functions (current rule) or all functions?
