# Conformance tests

Executable acceptance tests for the scenarios in `openspec/specs/` (and in active changes under
`openspec/changes/*/specs/`). They are written **from the scenarios only**, never from the
implementation, and are run against the real `aval` binary by `crates/aval-cli/tests/conformance.rs`.

## Layout

```
tests/conformance/<capability>/<ID>-<slug>.aval          single-file test
tests/conformance/<capability>/<ID>-<slug>/              multi-step test (spec lock)
    before.aval   after.aval
tests/conformance/<capability>/<ID>-<slug>.py            test needing Python tooling (run by CI only)
```

`<ID>` is the scenario ID (e.g. `TYP-001`). A file may cover several scenarios by listing extra IDs in a
`# scenario:` header; the file name carries the primary one. `scripts/trace.py` checks that every
scenario has at least one test and that every test names an existing scenario.

## Header

A test file starts with `#` comment lines (they are ordinary Aval comments, so the file stays valid input):

```
# scenario: TYP-001
# cmd: check
# exit: 1
# expect: {"code":"E301","expected":"Int","found":"Float"}
module t
fn f() -> Int
{
  return 1.0
}
```

| Key | Meaning | Default |
| --- | --- | --- |
| `scenario` | Space-separated scenario IDs covered | required |
| `cmd` | Arguments passed to `aval`; the token `$FILE` is replaced by the test file path. Shorthands: `check` = `check $FILE`, `test` = `test $FILE` | `check` |
| `exit` | Expected exit code | `0` if no `expect` line has an `E`/`T` code, else `1` |
| `expect` | A JSON object that must be a **subset** of one output line (every key present with an equal value; nested objects/arrays compared exactly). Lines are matched **in order**, and the output must contain **exactly** as many diagnostic lines as there are `expect` lines | none (= stdout must be empty) |
| `expect-codes` | Shorthand: space-separated codes, each one output line with that `code` | — |
| `stdout-contains` | Text that must appear in stdout (repeatable) | — |
| `stderr-contains` | Text that must appear in stderr (repeatable) | — |
| `no-file` | Do not substitute a real file (for `cmd: check does/not/exist.aval`) | — |
| `skip-output-count` | Do not enforce the exact line count (only for CLI tests printing non-diagnostics, e.g. `skill print`) | — |

Positions: `"at":[line,col]` may be included in `expect`; lines are counted in the whole file, header
included.

## Multi-step lock tests

A directory containing `before.aval` and `after.aval`. The runner copies `before.aval` to a temporary
directory as `main.aval`, runs `aval spec lock main.aval`, then (unless `after.aval` has
`# lock: none`) replaces it with `after.aval` and runs the `cmd` from `after.aval`'s header. A header
`# lock: none` in `before.aval` means no lock step (no `aval.lock` is created). A header
`# rename: <file name>` in `after.aval` writes it under that name instead of `main.aval` (and removes
`main.aval`); `$FILE` then refers to the new name. The header lives in `after.aval`.
