# E2: in-context evaluation, Aval vs Rust

Question: can a model that has never seen Aval write correct Aval with only the compiler-printed
reference (`aval skill print`) in context? The harness runs the same 40 HumanEval-derived tasks in Aval
and in Rust, drives a repair loop with compiler feedback, decides success with hidden tests, and
reports compile@1, pass@1, pass@R and tokens per solved task. Pre-registered criterion:
**Aval pass@1 >= 0.8 x Rust pass@1**.

Python 3 standard library only (`tiktoken` is optional, used for token counts). Needs the `aval` binary
and, for the Rust arm, `rustc` on `PATH`. Design: `openspec/changes/add-e2-harness/design.md`.

### Running with the Claude Code CLI

`providers/claude-cli.sh` wraps `claude -p` with no tools, no MCP servers, no settings and a neutral
system prompt, so the model only sees the conversation built by the harness:

```console
$ AVAL_E2_MODEL=<model id> python3 experiments/e2-in-context/harness.py run \
    --provider command --command "$PWD/experiments/e2-in-context/providers/claude-cli.sh" \
    --pass-env AVAL_E2_MODEL --aval-bin "$PWD/target/release/aval" --out out/
```

## Layout

```
harness.py            the harness (this file's subject)
tasks/<id>.json       one task per file (schema below)
reference/<id>.aval   reference answers: exactly the text a model would reply (one fenced block)
reference/<id>.rs
results.json          written by `run` (default: this directory)
RESULTS.md            written by `run`
```

Tasks are translated from HumanEval (MIT licence, https://github.com/openai/human-eval); every task
carries its attribution in `source`. MultiPL-E code and data are not used.

## Running

Build the compiler first (`cargo build`). The harness finds it through `--aval-bin`, then `$AVAL_BIN`,
then `target/debug/aval`, then `aval` on `PATH`, then `cargo run`.

```
# validate the task set (exit 0 only if every task is valid; prints "40 valid tasks")
python3 experiments/e2-in-context/harness.py validate-tasks

# sanity check: the reference solutions must solve everything (pass@1 = 1.0 in both languages)
python3 experiments/e2-in-context/harness.py run --provider replay --responses reference --out /tmp/e2-ref

# replay recorded answers
python3 experiments/e2-in-context/harness.py run --provider replay --responses answers.json --out /tmp/e2-replay

# any CLI that reads the prompt on stdin and prints the answer
python3 experiments/e2-in-context/harness.py run --provider command --command "my-model-cli" --out out/

# Anthropic Messages API (key from the environment, temperature 0)
ANTHROPIC_API_KEY=... python3 experiments/e2-in-context/harness.py run --provider anthropic --model <model-id> --out out/
```

Running the experiment with a real model needs credentials and a budget decision: it is not part of
this change.

### Reproducing the HumanEval+ import

`tools/import_humaneval_plus.py <test.jsonl>` regenerates the task files byte for byte from EvalPlus's
dataset, pinned to this revision and checksum (the data file is never committed):

```
curl -L -o test.jsonl https://huggingface.co/datasets/evalplus/humanevalplus/resolve/d32357cf319e50e9c8d8dab5ea876c72b0fd321b/test.jsonl
sha256sum test.jsonl   # 908377f1daf28dcb36846db73a5662b2e05a9907407c2696c89ad9d3b0b04492
python3 experiments/e2-in-context/tools/import_humaneval_plus.py test.jsonl
```

### Options of `run`

| Option | Meaning | Default |
| --- | --- | --- |
| `--provider replay\|command\|anthropic` | where answers come from | required |
| `--responses FILE\|reference` | replay: responses file, or the reference solutions | |
| `--command CMD` | command provider: shell command; prompt on stdin, answer on stdout | |
| `--model ID` | anthropic model id | |
| `--repairs R` | repair rounds after the first attempt (attempts = R + 1) | 3 |
| `--langs aval,rust` (`--lang`) | which arms to run | both |
| `--task ID` (`--only`) | run only these ids (repeatable, or comma separated) | all |
| `--tasks DIR` | task directory | `tasks/` next to `harness.py` |
| `--out DIR` | where `results.json` and `RESULTS.md` go | this directory |
| `--aval-bin CMD` | aval command | see above |
| `--tokenizer auto\|tiktoken\|estimate` | token counting when the provider reports none | auto |
| `--timeout S` | seconds per `aval`/`rustc`/test run | 60 |
| `--pass-env NAME` | command provider: also pass this environment variable (repeatable, or comma separated) | |

Exit code: 0 when the run completed (whatever the verdict), 2 on a harness/environment error
(missing binary, bad option, unreadable files), 1 from `validate-tasks` when a task is invalid.

The command provider runs the command in an **empty temporary working directory** with an environment
limited to `PATH`, `HOME`, `LANG`, the `AVAL_*` variables and those named by `--pass-env`, so it cannot read
`tasks/` or `reference/`. Use absolute paths in the command. The harness sets `AVAL_TASK_ID`, `AVAL_LANG`
and `AVAL_ATTEMPT`, so a script can answer per task (for example `cat /answers/$AVAL_TASK_ID.$AVAL_LANG`).
Only the `anthropic` provider is leak-free by construction; an agentic CLI with other credentials or file
access is the operator's responsibility.
Its stdin is the whole conversation: `[system]`, then alternating `[user]` / `[assistant]` blocks.

## Task file (`tasks/<id>.json`)

```json
{
  "id": "HE-000",
  "source": "HumanEval/0 (MIT, https://github.com/openai/human-eval)",
  "statement": "English description, no Aval or Rust code",
  "aval": { "name": "has_close_elements",
            "params": [["numbers", "List<Float>"], ["threshold", "Float"]],
            "returns": "Bool" },
  "rust_signature": "fn has_close_elements(numbers: Vec<f64>, threshold: f64) -> bool",
  "hidden": [ { "args": [[1.0, 2.0, 3.9], 0.3], "expected": true } ],
  "reference": { "aval": "experiments/e2-in-context/reference/HE-000.aval",
                 "rust": "experiments/e2-in-context/reference/HE-000.rs" }
}
```

- The file name is `<id>.json`. Reference paths are relative to the repository root (a path that
  does not exist there is also looked up in `reference/` next to the task directory).
- Supported types (Aval syntax, used for conversion in both languages): `Int`, `Float`, `Bool`,
  `String`, `List<T>`, `Result<T, E>`. Values are JSON; `Result` is `{"Ok": v}` / `{"Err": e}`; a
  `Float` may be written as a JSON integer. Strings may contain only characters Aval can escape
  (`\"`, `\\`, newline, tab among the control characters).
- `rust_signature` must declare `fn <aval.name>` with as many parameters as `aval.params`. Parameters
  of type `&str`, `&[T]` and `&T` are supported (the harness passes a literal / a reference to a
  `vec!`); `&mut` is not.
- `source_tests` (string mentioning `HumanEval+` or `MBPP+`; it names the task set) and `excluded` (object, counts per excluded reason) are
  required; `atol` (number >= 0, default 0) is the tolerance of the HumanEval+ cases. A Float is compared as `abs(got - expected) <= atol + 1e-7 * abs(expected)` (EvalPlus's
  `atol` and `rtol`) when `atol > 0` (element-wise inside lists and `Result`), and exactly (`==`) when `atol` is 0 or absent
  (EvalPlus semantics; `approx` is not used for hidden cases).
- At least 20 hidden cases. The statement must not contain code fences and must not show hidden cases
  (the harness never adds them to a prompt, but a statement that quotes them would leak).
- `validate-tasks` also checks that each reference file contains a fenced block of its language.

## How an attempt is judged

1. **Prompt.** System prompt for Aval = the exact output of `aval skill print` (SHA-256 recorded as
   `skill_sha256`); for Rust a one-line Rust system prompt. User prompt = statement, required
   signature, and the rule (one `pub fn` with `ensures` or `examples` and a `cost`, in a single
   ```` ```aval ```` block; Rust: a single ```` ```rust ```` block). Hidden cases are never shown.
2. **Extraction.** The first fenced block of the expected language. None: attempt fails with reason
   `no_code`.
3. **Aval.** `aval check` on the candidate alone; any `E` code or non-zero exit is `compile_error`
   (diagnostics are sent back verbatim). Then the harness appends the hidden cases in **batches** of
   50: `fn hidden_b<k>() -> Int` with `examples => -1` calls the function for each case, compares (Floats
   with the task's `atol` (and EvalPlus's relative term), lists and `Result` element by element) and returns the index of the first
   failing case, or -1; then `aval test --max-steps 500000000` runs (a batch of 50 cases gets 50 times the default 10M-step
   per-input budget, so batching never gives a case less budget than it would have alone; the per-case
   rerun below uses the default budget). A `T100`/`T700` on a batch function sends the harness back
   to run that batch's cases one function each (`hidden_<i>`), which gives the exact failing indexes.
   An `E` code on a hidden function is a signature problem and fails every case; a non-zero exit without any
   diagnostic is also a failure (positive attestation: a silent run is only trusted with exit code 0).
   Because `approx`, `print`, `Ok` and `Err` cannot be redefined (`E102`), the hidden tests' use of
   `approx` cannot be subverted. Diagnostics of the
   candidate's own spec (`ensures`, `examples`) are sent back and counted, but only hidden cases
   decide success.
4. **Rust.** `rustc --edition 2021 -A warnings` on the candidate alone (as a library); errors are sent
   back (paths scrubbed). Then a generated `main` holds the cases as a static table of non-capturing closures, runs each under
   `catch_unwind` in a loop and prints, per batch of 50, `HIDDEN_BATCH <b> <nonce> <bits>` (one `0`/`1` per
   case) and a final `HIDDEN_DONE <nonce>` (the nonce is random per attempt); a case counts as passed
   only if its batch line says `1` and `HIDDEN_DONE` is present, so `std::process::exit(0)` or a
   missing line fails. The binary runs from a fresh empty directory after the sources are deleted. This
   attestation guards against accidental early exit and stray output; it is **not a sandbox**: a
   malicious candidate could still read the nonce from its own binary or read the task files from the
   host. Resistance to deliberate cheating needs a container or VM. `rustc -O -C overflow-checks=on` is used for
   the test binary, so integer overflow fails in both languages. A candidate that compiles alone but not with the harness (wrong name or
   parameter types, its own `main`) gets a generic signature message, never the generated source.
5. **Repair.** After a failed attempt the harness replies with the diagnostics and the *count* of
   failing hidden cases (never the cases), up to `--repairs R` times. A task is solved at the first
   attempt where every hidden case passes.

Timeouts (default 60 s per command) fail the attempt with reason `timeout`. A crash (any exit code other
than 0/1 from `aval` or `rustc`, a signal or non-zero exit of the Rust test binary, a compiler ICE) fails
it with reason `crash`; the run continues. `results.json` and `RESULTS.md` are rewritten after every task.

A provider failure (HTTP error, command exiting non-zero, timeout, missing replay entry) gives the task
`outcome` = `provider_error`: it is excluded from the metrics and the verdict is `INCONCLUSIVE` while any such
task exists (rerun those tasks, for example with `--task`, in a separate `--out`). An empty model reply is
sent back to the model as `(empty reply)`.

## Replay file

```json
{ "HE-000": { "aval": ["answer 1", "answer 2"], "rust": ["answer 1"] } }
```

Attempt `n` replays entry `n`; attempts beyond the list repeat the last entry. A task or language
missing from the file ends that task with reason `no_response`. `--responses reference` replays the
reference files for every attempt.

## Outputs

`results.json` (keys sorted; the only non-deterministic field is `generated_at`):

```
schema, generated_at, provider, model, skill_sha256, repairs, languages, task_count,
metrics.<lang>: tasks, compile_at_1, pass_at_1, pass_at_r, solved, pass_at_1_count,
                tokens_in, tokens_out, tokens_per_solved, token_sources, unsolved_reasons
verdict: criterion, aval_pass_at_1, rust_pass_at_1, threshold, result (PASS|FAIL|INCONCLUSIVE), degenerate
tasks.<id>.<lang>: outcome (solved|failed|provider_error), resource_failure, solved, solved_at, compile_at_1, pass_at_1, failed_cases, reason,
    attempts[]: attempt (1-based), prompt (the message sent this turn), answer, outcome (solved|failed|provider_error),
                reason (null | no_code | compile_error | hidden_failed | crash | timeout | no_response |
                provider_error | provider_timeout | provider_http_<code>),
                compile_ok, diagnostics (count), hidden_total, failed_cases (0-based indexes),
                fault_kinds (T700 faults seen on hidden cases: step_limit | memory_limit | stack_limit |
                overflow | other; Aval only), value_failures (cases failing with a wrong value),
                tokens_in, tokens_out, tokens_system, tokens_source (provider | tiktoken | estimate)
```

Metric definitions: `compile_at_1` = share of tasks whose first answer passes `aval check` (Rust:
compiles); `pass_at_1` = first answer passes every hidden case; `pass_at_r` = solved within R repairs;
`resource_failures` = tasks whose final attempt failed only on `step_limit`/`memory_limit`/`stack_limit`
faults (no wrong values; reported as the `resource failures` row of `RESULTS.md`, so interpreter limits
can be told apart from wrong answers); `tokens_per_solved` ("with system") = all tokens (prompt + answer) of every attempt of every task, divided by
the number of solved tasks; `tokens_per_solved_without_system` subtracts the system prompt of every attempt
(the Aval arm pays the whole ~2.4k-token reference each time, Rust a one-line prompt), so report both.
The decision criterion is pass@1 only; tokens are a secondary cost figure. Token counts come from the provider when it reports usage; otherwise `tiktoken`
(`o200k_base`) if installed and loadable, else `ceil(chars / 4)`; the source is recorded per attempt.

`RESULTS.md` holds the table of the metrics per language, a per-task table, and a line
`Verdict: PASS|FAIL (Aval pass@1 = x, Rust pass@1 = y, needs >= z)`.

## Task sets and confidence intervals

`--tasks DIR` selects the task set: `tasks/` (HumanEval+) or `tasks-mbpp/` (MBPP+). The set is inferred
from the tasks' `source_tests` and named in `results.json` (`task_set`) and in `RESULTS.md`
(`Task set: HumanEval+` or `Task set: MBPP+`; `mixed` if a directory mixes them). Runs of different sets
are separate.

`RESULTS.md` (and `results.json` under `confidence`) reports 95 % percentile bootstrap intervals over
tasks, 10,000 resamples with a fixed seed (standard library `random`, so reruns are identical): one line
`<Lang> pass@1 95% CI: [low, high]` per language and, when both languages ran, `Aval − Rust 95% CI`
for the paired difference (the same resamples are used for both languages). Tasks with provider errors
are left out. The verdict criterion is unchanged; read it together with the intervals.

## Threats to validity

- **Feedback asymmetry.** After a hidden failure the Aval arm also receives diagnostics from the candidate's
  own `examples`/`ensures` (that is the language under test), while the Rust arm gets only the failing
  count; Aval compile@1 includes spec rules (`cost`, `ensures`/`examples` on `pub`) that `rustc` lacks.
  The system prompts differ in size (see the two token figures).
- **HumanEval contamination.** The statements were rewritten but the tests and the canonical problems are
  likely in the training data of current models, which favours Rust (and Python-like thinking) over a
  language the model has never seen; a high Rust pass@1 may partly be recall.
- **Statistical power.** One sample at temperature 0 per task; a single task moves pass@1 by 1/N. The
  bootstrap intervals cover task sampling only, not model sampling noise. Treat a verdict near the
  threshold, or whose interval straddles it, as inconclusive.
- **Unsandboxed Rust.** Model-written Rust is compiled and run on the host with only a timeout; run the
  harness in a container or VM. Aval candidates run in the interpreter, which has no I/O.
- **Interpreter limits.** Aval answers run in a tree-walking interpreter with a 10M-step budget per input
  (a batch of k cases gets k times that), 10,000 nested calls (`stack_limit`) and 10M-element values
  (`memory_limit`). A correct but naive or deeply recursive answer can fail on these where Rust would
  not (Rust has native speed and a platform stack). The `resource failures` row counts such tasks so
  the verdict can be read with and without them; the Rust arm does not report fault kinds.
- **Task translation.** Two hidden cases depart from upstream HumanEval where the statement defines the
  behaviour (HE-044 `x=0`, HE-064 empty word); see `tasks/ATTRIBUTION.md`.
