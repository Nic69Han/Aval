# eval-harness Specification

## Purpose
Defines the harness for experiment E2, which measures how well a model writes Aval from the in-context
reference alone, compared with Rust on the same tasks.

## Requirements

### Requirement: Task set validation
`harness.py validate-tasks` SHALL check every task file against the schema (id, source, statement, Aval and Rust signatures, at least 5 hidden cases, reference solutions) and exit 0 only if all are valid.

#### Scenario: EVH-001 Shipped task set is valid
- **WHEN** `python3 experiments/e2-in-context/harness.py validate-tasks` runs on the shipped task set
- **THEN** it reports 40 valid tasks and exits with code 0

#### Scenario: EVH-002 Reference solutions solve every task
- **WHEN** the harness runs all tasks with `--provider replay --responses reference` for both languages
- **THEN** pass@1 is 1.0 for Aval and for Rust

### Requirement: Hidden tests decide success
A task SHALL be solved only if the final answer passes every hidden case; hidden cases SHALL NOT appear in any prompt.

#### Scenario: EVH-003 Wrong answer fails
- **WHEN** a replayed Aval answer compiles but returns a wrong value on one hidden case
- **THEN** the task is recorded as not solved with the failing case index

#### Scenario: EVH-004 Hidden cases never reach the model
- **WHEN** the harness runs any task with the replay provider
- **THEN** no recorded prompt contains the text of any hidden case's expected value or call

### Requirement: Repair loop
After a failed attempt the harness SHALL send the diagnostics (Aval JSON lines or rustc errors, plus failing hidden-case count without the cases) back to the model, up to `--repairs R` times (default 3).

#### Scenario: EVH-005 Repaired on the second attempt
- **WHEN** the first replayed answer triggers `E301` and the second is correct
- **THEN** the task is solved at attempt 2, `compile_at_1` is false and `pass_at_1` is false

#### Scenario: EVH-006 Repair budget
- **WHEN** every replayed answer is wrong and `--repairs 2` is set
- **THEN** exactly 3 attempts are recorded and the task is not solved

### Requirement: Prompt contents
The system prompt SHALL be exactly the output of `aval skill print`, whose SHA-256 is recorded in the results.

#### Scenario: EVH-007 Skill hash recorded
- **WHEN** a run completes
- **THEN** `results.json` contains `skill_sha256` equal to the SHA-256 of `aval skill print` output

### Requirement: Code extraction
The harness SHALL take the first fenced block of the expected language; an answer without one is a failed attempt with reason `no_code`.

#### Scenario: EVH-008 Answer without code block
- **WHEN** a replayed answer contains no fenced block
- **THEN** the attempt is recorded with `reason` = `no_code`

### Requirement: Results and verdict
The harness SHALL write `results.json` (per task and per attempt: tokens in/out with their source, diagnostics count, outcome) and `RESULTS.md` with compile@1, pass@1, pass@R and tokens per solved task for each language, and the verdict of the criterion Aval pass@1 ≥ 0.8 × Rust pass@1.

#### Scenario: EVH-009 Verdict line
- **WHEN** a run of both languages completes
- **THEN** `RESULTS.md` contains a line starting with `Verdict:` followed by `PASS` or `FAIL` and the two pass@1 values

### Requirement: Rust baseline
Rust answers SHALL be compiled with `rustc` and checked against the same hidden cases.

#### Scenario: EVH-010 Rust compile error is fed back
- **WHEN** the first replayed Rust answer does not compile and the second is correct
- **THEN** the task is solved at attempt 2 in the Rust arm

### Requirement: Providers
The harness SHALL support `replay` (responses from a JSON file keyed by task, language and attempt), `command` (a shell command receiving the prompt on stdin and printing the answer) and `anthropic` (Messages API, key from `ANTHROPIC_API_KEY`, temperature 0).

#### Scenario: EVH-011 Command provider
- **WHEN** the harness runs one task with `--provider command --command "cat reference_answer.txt"`
- **THEN** the task is solved and the attempt's token source is not `provider`

#### Scenario: EVH-012 Replay is deterministic
- **WHEN** the same replay run is executed twice
- **THEN** both `results.json` files are identical except for timestamps

### Requirement: Positive attestation of hidden cases
A hidden case SHALL count as passed only if the run explicitly reports it as passed (Rust: a `HIDDEN_OK <i>` line per case and a final `HIDDEN_DONE` line; Aval: `aval test` exit 0 and no diagnostic on any `hidden_*` function, with every `hidden_*` function present in the checked module); anything else is a failure.

#### Scenario: EVH-013 Rust candidate exits early
- **WHEN** a replayed Rust answer calls `std::process::exit(0)` inside the function
- **THEN** the task is not solved

#### Scenario: EVH-014 Aval candidate redefines approx
- **WHEN** a replayed Aval answer for a Float task defines `fn approx(a: Float, b: Float) -> Bool` returning `true` and a wrong body
- **THEN** the task is not solved

### Requirement: Crashes and timeouts are failed attempts
A crash (any exit code other than 0 or 1 from `aval`, a non-zero exit or signal from the Rust binary, or a compiler crash) or a timeout (default 60 s per command) SHALL be a failed attempt with `reason` = `crash` or `timeout`, never an aborted run; `results.json` SHALL be rewritten after every task.

#### Scenario: EVH-015 Aborting candidate
- **WHEN** a replayed Aval answer makes `aval test` exit with code 134
- **THEN** the attempt has `reason` = `crash`, the run continues and `results.json` is written

### Requirement: Provider errors make the verdict inconclusive
A task whose provider call failed SHALL be recorded with `outcome` = `provider_error` and excluded from the metrics, and the verdict SHALL be `INCONCLUSIVE` if any such task exists.

#### Scenario: EVH-016 Failing provider
- **WHEN** the command provider exits with code 1 for one task
- **THEN** that task has `outcome` = `provider_error` and `RESULTS.md` contains `Verdict: INCONCLUSIVE`

### Requirement: Isolation of the command provider
The command provider SHALL run in an empty temporary working directory with an environment limited to `PATH`, `HOME`, `LANG`, the `AVAL_*` variables and variables named by `--pass-env`.

#### Scenario: EVH-017 Command cannot see the task files
- **WHEN** the command provider runs `ls; cat tasks/*.json`
- **THEN** the recorded answer contains no task file content

### Requirement: Fair accounting
`RESULTS.md` SHALL report tokens per solved task both with and without the system prompt, and the Rust arm SHALL compile with overflow checks enabled (`-O -C overflow-checks=on`) so integer overflow fails in both languages.

#### Scenario: EVH-018 Both token figures
- **WHEN** a run of both languages completes
- **THEN** `RESULTS.md` contains `tokens/solved (with system)` and `tokens/solved (without system)` for each language

### Requirement: Test binary runs without its sources
The Rust test binary SHALL run in an empty temporary directory that does not contain its source or the nonce; the documentation SHALL state that attestation guards against accidental early exit and stray output, and is not a sandbox against a malicious candidate.

#### Scenario: EVH-019 Candidate reads its own source to forge attestation
- **WHEN** a replayed Rust answer with a wrong body reads `main.rs` from its working directory and prints the attestation lines it finds there
- **THEN** the task is not solved

### Requirement: Degenerate comparisons are inconclusive
When Rust pass@1 is 0, the verdict SHALL be `INCONCLUSIVE`.

#### Scenario: EVH-020 Both arms fail everything
- **WHEN** every replayed answer in both languages contains no code
- **THEN** `RESULTS.md` contains `Verdict: INCONCLUSIVE`

### Requirement: Hidden cases come from HumanEval+
Every shipped task SHALL take its hidden cases from an EvalPlus dataset (`source_tests` names HumanEval+ or MBPP+ and the licence), keep every case expressible with Aval's types, and record the excluded cases as counts per reason in `excluded`.

#### Scenario: EVH-021 Task files record their HumanEval+ provenance
- **WHEN** `harness.py validate-tasks` runs on the shipped HumanEval+ task set
- **THEN** every task has `source_tests` mentioning `HumanEval+`, an `excluded` object, and at least 20 hidden cases, and the command exits with code 0

### Requirement: Float tolerance follows EvalPlus
A hidden case with `atol > 0` SHALL pass when `abs(got - expected) <= atol + 1e-7 * abs(expected)` (EvalPlus `assert_allclose` semantics, element-wise for lists) and fail otherwise; with `atol == 0` the comparison is exact.

#### Scenario: EVH-022 Answer within tolerance
- **WHEN** a replayed answer for a task with `atol` = 1e-6 returns values off by 1e-8 on every case, and another replayed answer is off by 1e-3
- **THEN** the first task run is solved and the second is not

### Requirement: Batched hidden checks
Hidden cases SHALL be executed in batches so that a task with 1,000 cases is checked within the per-command timeout, with attestation per batch.

#### Scenario: EVH-023 Large hidden set
- **WHEN** the reference answers of all 40 tasks are replayed for both languages
- **THEN** every task is solved and no attempt has reason `timeout`

### Requirement: Import is reproducible
`tools/import_humaneval_plus.py <test.jsonl>` SHALL regenerate the shipped task files byte for byte.

#### Scenario: EVH-024 Re-import
- **WHEN** the import script runs on the EvalPlus `test.jsonl` into a temporary directory
- **THEN** the produced task files are identical to the shipped ones

### Requirement: Per-case step budget in batches
A batch of k hidden cases SHALL run with a step budget of k times the per-input budget, so batching never gives a case less budget than it would have alone.

#### Scenario: EVH-025 Slow but correct answer is not penalised by batching
- **WHEN** a replayed Aval answer that needs about 3 million steps per case is run on a task whose batches hold 50 cases
- **THEN** the task is solved

### Requirement: Resource failures are reported separately
For every attempt the harness SHALL record the fault kinds seen on hidden cases (`step_limit`, `memory_limit`, `overflow`, other), and `RESULTS.md` SHALL report, per language, the number of tasks whose final attempt failed only because of `step_limit` or `memory_limit`.

#### Scenario: EVH-026 Resource failure is visible
- **WHEN** a replayed Aval answer exceeds the step budget on some hidden cases and is otherwise correct
- **THEN** the attempt records `fault_kinds` containing `step_limit` and `RESULTS.md` contains a `resource failures` row

### Requirement: MBPP+ task set
The repository SHALL ship an MBPP+ task set in `experiments/e2-in-context/tasks-mbpp/` with at least 150 valid tasks, every one with Aval and Rust references; every MBPP+ task whose values are expressible but which is not shipped SHALL be listed in `tasks-mbpp/EXCLUDED.md` with a reason.

#### Scenario: EVH-027 MBPP+ set is valid and solved by its references
- **WHEN** `harness.py validate-tasks --tasks experiments/e2-in-context/tasks-mbpp` runs and the references are replayed for both languages
- **THEN** at least 150 tasks are valid, every `source_tests` mentions `MBPP+`, and pass@1 is 1.0 for both languages

### Requirement: Reproducible MBPP+ import
`tools/import_mbpp_plus.py` SHALL refuse any input whose SHA-256 differs from the pinned EvalPlus file and SHALL regenerate the shipped MBPP+ task files byte for byte (references excepted).

#### Scenario: EVH-028 Re-import MBPP+
- **WHEN** the importer runs on the pinned parquet file into a temporary directory
- **THEN** the produced task files equal the shipped ones, and a modified file is refused

### Requirement: Visible example is not a hidden case
Each MBPP+ statement SHALL contain exactly one `Example:` line, and that case SHALL NOT appear among the task's hidden cases.

#### Scenario: EVH-029 Example excluded from hidden
- **WHEN** every MBPP+ task file is inspected
- **THEN** its statement has one `Example:` line whose arguments and value match no hidden case, and `excluded.shown_in_statement` is 1

### Requirement: Results name their task set
`results.json` and `RESULTS.md` SHALL name the task set (`HumanEval+` or `MBPP+`) used by the run.

#### Scenario: EVH-030 Task set in the report
- **WHEN** a replay run on the MBPP+ set completes
- **THEN** `RESULTS.md` contains `Task set: MBPP+` and `results.json` has `task_set` = `MBPP+`

### Requirement: Confidence intervals
`RESULTS.md` SHALL report a 95 % bootstrap confidence interval for each language's pass@1 and for the paired difference Aval − Rust (10,000 resamples over tasks, fixed seed, so reruns give identical intervals).

#### Scenario: EVH-031 Intervals are reported and deterministic
- **WHEN** the same replay run is executed twice
- **THEN** both `RESULTS.md` files contain identical lines `pass@1 95% CI` for each language and `Aval − Rust 95% CI`

### Requirement: Statements are language-neutral and signatures follow the original tests
MBPP+ statements SHALL NOT name a programming language, and each inferred signature SHALL match the argument shapes of the original MBPP `test_list` (ties in type inference are broken by that shape, never alphabetically).

#### Scenario: EVH-032 Neutral statements and faithful signatures
- **WHEN** every MBPP+ task file is inspected
- **THEN** no statement contains the word `python` (any case), and every parameter's nesting depth equals the depth of the corresponding argument in the first original MBPP assertion
