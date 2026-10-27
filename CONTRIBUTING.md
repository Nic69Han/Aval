# Contributing to Aval

Thank you for your interest in Aval. This guide explains how to propose changes.

## Ground rules

- All code, documentation, issues and reviews are written in **English**.
- Be respectful: the [Code of Conduct](CODE_OF_CONDUCT.md) applies everywhere.
- Sign off your commits (`git commit -s`) to certify the
  [Developer Certificate of Origin](https://developercertificate.org/).
- AI-assisted contributions are welcome. Please say so in the pull request; you remain responsible
  for the change.

## Specification first

Aval uses [OpenSpec](https://github.com/Fission-AI/OpenSpec) for spec-driven development:

- `openspec/specs/` holds the current requirements. Each scenario has a stable ID such as `TYP-001`.
- `openspec/changes/<change-id>/` holds proposals: `proposal.md`, `design.md`, `tasks.md` and spec deltas.
- `tests/conformance/` holds executable tests. Each test names the scenario IDs it covers.

**Changes to language behavior** start with an OpenSpec change proposal, opened as a pull request
for discussion. **Bug fixes** that make the implementation match an existing scenario only need to
cite the scenario ID.

## Development

```console
$ cargo fmt --all
$ cargo clippy --workspace --all-targets -- -D warnings
$ cargo test --workspace
$ openspec validate --all --strict
$ python3 scripts/trace.py        # every scenario has a test, every test names a scenario
```

Conformance tests are described in [tests/conformance/README.md](tests/conformance/README.md).

## Contributor License Agreement

Before your first pull request, read [CLA.md](CLA.md) and explicitly agree to it in the pull request description or a comment by writing: `I have read the Aval CLA and agree to its terms.` This agreement applies to the contribution you submit; it does not require you to publish or report private changes that you do not submit to Aval. The CLA does not transfer ownership of your copyright in your contribution.

## Pull requests

- Keep pull requests small and focused on one change.
- Make sure CI passes.
- Changes to locked specifications (`aval.lock`) and to accepted requirements need a maintainer's
  approval.

## Getting help

Open a [discussion or issue](https://github.com/Nic69Han/Aval/issues) if anything is unclear.
