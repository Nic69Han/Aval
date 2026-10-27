# Run 01 — notes

- Provider: `providers/claude-cli.sh` (Claude Code CLI, no tools, no MCP, no settings, neutral system
  prompt), one model for both arms, default sampling of the CLI.
- 40 tasks × 2 languages, up to 3 repairs. Tokens counted with o200k (`tiktoken`), not provider usage.
- Verdict: **PASS** — Aval pass@1 = 0.950, Rust pass@1 = 1.000 (criterion: ≥ 0.8 × Rust).
  Aval compile@1 = 0.950; pass@3 = 1.000 in both languages.

## The two Aval first-attempt failures

Both were rejected by `aval check` and repaired from the diagnostics:

| Task | Diagnostic | Cause |
| --- | --- | --- |
| HE-046 | `E204` `(0..n).fold` on `Range<Int>`, `avail: []` | the model used a Rust iterator method on a range |
| HE-076 | `E002` unexpected `then`; then `E204` `(0..64).fold` | same range habit after a syntax slip |

Lesson for the language: ranges are not iterable values with methods in v0.1, while models expect
Rust's `(a..b).fold(...)`. Candidate follow-up change: give `Range<Int>` the list methods (or make
`a..b` a `List<Int>` in expression position), and let `E204` list the available members instead of `[]`.

## Threats to validity

See the harness README. In particular: HumanEval is likely in the model's training data (which helps
Rust more than Aval), a single run without confidence intervals, and the Aval arm pays the whole
reference (~2,400 tokens) on every attempt: 2,931 tokens per solved task with the system prompt,
343 without, versus 220 / 193 for Rust.
