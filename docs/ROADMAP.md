# Roadmap

This roadmap gives direction, not dates. Each item becomes an OpenSpec change when work starts.

## Now — core toolchain

- Reference checker and interpreter: `aval check`, `aval test`, spec lock, `aval skill print`
- Conformance suite covering every scenario

## Next

- Evaluation harness: how well models write Aval with only the reference in context
- Grammar export for constrained decoding (GBNF / Lark)
- Linear resources (`resource`, `using`)
- Collections (`Set`, `Map`, bounded `Queue`)
- Constructive input generation and spec-strength (mutation) analysis
- Cost checking and memory checks

## Later

- Dimension types (`Image`, `Tensor`)
- Native compilation through a Rust backend
- Editor support (LSP, tree-sitter grammar)
- Optional proof level with SMT solvers
- Formal model of the core language
