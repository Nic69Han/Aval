# aval-eval

Tree-walking reference interpreter and the `aval test` runner (examples, property tests derived from
`ensures` under `requires`, oracle comparison, runtime faults, shrinking).

## D6 decision: in-house generator and shrinker, not `proptest`

Design D6 allowed either `proptest` or a ~300-line in-house generator/shrinker if `proptest` could not meet
TST-008/009/010 exactly. We built it in-house (`src/gen.rs`, about 500 lines with tests) for these reasons:

- **Exact, documented shrinking.** TST-009 requires the minimal counterexample `{"x": 100}` and TST-010
  requires `{"xs": [7]}`. Our integer shrinker walks toward 0 by binary search (0, then `a - a/2`, `a - a/4`, ...,
  `a - 1`), lists remove elements before shrinking them (plus an adjacent-sum merge so `[3, 4]` reaches `[7]`),
  floats go toward `0.0` and then integral values. With `proptest`, the result would depend on its internal
  `ValueTree` simplification order and on how filtered (`requires`) values are re-sampled.
- **Determinism (TST-008).** One tiny splitmix64 generator seeded from `--seed` (default 0) and an FNV-1a hash of
  the function name. Output never depends on hash-map iteration order or on library-internal RNG changes.
- **Dynamic values.** Inputs are `Value`s driven by the checked parameter types, not Rust types; `proptest`
  strategies are compile-time typed, so we would have needed `BoxedStrategy<Value>` wrappers around hand-written
  logic anyway, with a dependency tree (`rand`, `regex-syntax`, ...) that buys nothing here.
- **Rejection and edge cases.** `requires` filtering is plain rejection sampling (at most 10x the case count),
  and the fixed edge-case battery runs before random cases; both are simpler to control directly.

How a run works, per function: examples first (and each example is also checked against `ensures`), then the
edge-case battery, then `--cases` accepted random inputs. All inputs are evaluated, the simplest failing input of
each kind (`T200`, `T600`, `T700`) is kept, and only that one is shrunk with the predicate "same kind of failure
still occurs and `requires` still holds". Extreme scalars (`i64::MIN/MAX`, `1e308`) are generated inside lists, struct fields and variant payloads too
(low rate in random cases, plus singleton/field/payload edge cases), so overflow on compound inputs is found.
Recursive types are generated through least construction ranks (`aval-check` rejects uninhabited types with
`E312`), so generation always terminates. If fewer than 10% of at least 10 requested cases satisfy `requires`
the property is reported as `T210`. The adjacent-integer merge in list shrinking is a sum-oriented heuristic;
shrinking is greedy and local, so counterexamples are small but not guaranteed globally minimal.

Oracle faults are reported as `T600` with `oracle: null` and `oracle_fault: <name>`; the body's own faults are
`T700`. Faulting `examples` expressions are `T700` with `fault` and `input`.

Numeric notes: `approx(a, b)` is `abs(a-b) <= 1e-9 * max(1, abs(a), abs(b))`, and identical values (including
infinities) are always approximately equal. Runtime fault names: `overflow`, `division_by_zero`,
`unwrap_on_err`, `empty_list` (`min`/`max` on an empty list), `index_out_of_bounds` (list index or slice),
`negative_exponent` (`Int.pow` with a negative exponent), `memory_limit` (a String or List over 10,000,000 bytes/elements), `step_limit` (more than 10 million evaluation
steps for one input, adjustable with `aval test --max-steps N`), `stack_limit` (more than 10,000 nested user calls). Per function, generation stops after 60M steps and
shrinking after 60M steps, so exponential programs still finish quickly. `print` output is discarded during tests.
