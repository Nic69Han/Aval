# Aval

**A programming language designed to be written by AI agents and verified by its compiler.**

Aval accepts code only when it type-checks, satisfies its specification, declares its effects and
stays within its declared cost. Function bodies read like simplified Rust; contracts, effects and
cost live in the signature.

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
```

```console
$ aval check stats.aval   # syntax, types, effects, spec rules
$ aval test stats.aval    # examples, property-based tests, minimal counterexamples
```

> **Status: pre-alpha.** The language and tools are under active development and will change.

## Highlights

- **Everything in the signature** — types, contract, effects, allocation and cost.
- **Locked specifications** — a spec cannot be weakened to make an implementation pass.
- **Self-testing** — examples and property-based tests run from the contract.
- **Machine-readable diagnostics** — one JSON object per line.
- **Small and familiar** — 46 keywords, Rust-like syntax, a reference that fits in a prompt
  (`aval skill print`).

## Getting started

Aval currently builds from source with a recent stable Rust toolchain:

```console
$ git clone https://github.com/Nic69Han/Aval
$ cd Aval
$ cargo build --release
$ ./target/release/aval --help
```

## Documentation

- [Language reference](docs/spec/aval-spec-v0.1.md)
- [Specifications and change proposals](openspec/) (requirements with testable scenarios)
- [Conformance tests](tests/conformance/)
- [Roadmap](docs/ROADMAP.md)

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Report security issues as described in [SECURITY.md](SECURITY.md).

## License

Aval is licensed under the [Apache License 2.0](LICENSE). Contributions are covered by the [Aval Contributor License Agreement](CLA.md). The CLA lets the project maintainers use and, where appropriate, relicense submitted contributions; it does not require contributors to publish private modifications or notify the project about changes they do not submit.
